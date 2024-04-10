use std::mem;

use super::Compose;
use bevy::prelude::*;

pub fn flex<C: Compose>(content: C) -> Flex<C> {
    Flex {
        content,
        on_click: None,
    }
}

pub struct Flex<C> {
    content: C,
    on_click: Option<Box<dyn FnMut(&mut World) + Send + Sync>>,
}

impl<C> Flex<C> {
    pub fn on_click<Marker>(mut self, system: impl IntoSystem<(), (), Marker>) -> Self {
        let mut cell = Some(IntoSystem::<(), (), Marker>::into_system(system));
        let mut id_cell = None;
        self.on_click = Some(Box::new(move |world| {
            if let Some(system) = cell.take() {
                let id = world.register_system(system);
                id_cell = Some(id);
            }

            let id = id_cell.unwrap();
            world.run_system(id).unwrap();
        }));
        self
    }
}

impl<C: Compose> Compose for Flex<C> {
    type State = (Entity, C::State);

    fn build(&mut self, world: &mut World, children: &mut Vec<Entity>) -> Self::State {
        let parent_children = mem::take(children);
        let content_state = self.content.build(world, children);
        let my_children = mem::replace(children, parent_children);

        let mut entity = world.spawn(ButtonBundle {
            style: Style {
                width: Val::Px(150.0),
                height: Val::Px(65.0),
                border: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(Color::BLACK),
            ..default()
        });
        entity.push_children(&my_children);

        let id = entity.id();
        children.push(id);

        if let Some(handler) = self.on_click.take() {
            entity.insert(ClickHandler {
                handler: Some(handler),
            });
        }

        (id, content_state)
    }

    fn rebuild(
        &mut self,
        target: &mut Self,
        state: &mut Self::State,
        world: &mut World,
        children: &mut Vec<Entity>,
    ) {
        let parent_children = mem::take(children);
        self.content
            .rebuild(&mut target.content, &mut state.1, world, children);
        let _my_children = mem::replace(children, parent_children);

        children.push(state.0);
    }
}

#[derive(Component)]
pub struct ClickHandler {
    handler: Option<Box<dyn FnMut(&mut World) + Send + Sync>>,
}

pub fn handler_system(world: &mut World) {
    let mut query =
        world.query_filtered::<(&Interaction, &mut ClickHandler), Changed<Interaction>>();

    let mut handlers: Vec<_> = query
        .iter_mut(world)
        .map(|(interaction, mut handler)| (*interaction, handler.handler.take()))
        .collect();

    for (interaction, f) in &mut handlers {
        match interaction {
            Interaction::Pressed => {
                if let Some(ref mut f) = f {
                    f(world)
                }
            }
            _ => {}
        }
    }

    for (idx, (_, mut handler)) in query.iter_mut(world).enumerate() {
        handler.handler = handlers[idx].1.take();
    }
}
