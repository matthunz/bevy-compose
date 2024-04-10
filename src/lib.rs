use bevy::{
    ecs::{
        component::Component,
        entity::Entity,
        query::{Changed, With},
        system::IntoSystem,
        world::World,
    },
    hierarchy::BuildWorldChildren,
    render::color::Color,
    text::{Text, TextSection, TextStyle},
    ui::{
        node_bundles::{ButtonBundle, NodeBundle, TextBundle},
        AlignItems, BackgroundColor, FlexDirection, Interaction, JustifyContent, Style, UiRect,
        Val,
    },
    utils::default,
};
use std::{any::Any, mem};

pub trait Compose: Send + Sync + 'static {
    type State: Send + Sync + 'static;

    fn build(&mut self, world: &mut World, children: &mut Vec<Entity>) -> Self::State;

    fn rebuild(
        &mut self,
        target: &mut Self,
        state: &mut Self::State,
        world: &mut World,
        children: &mut Vec<Entity>,
    );
}

impl Compose for () {
    type State = ();

    fn build(&mut self, _world: &mut World, _children: &mut Vec<Entity>) -> Self::State {}

    fn rebuild(
        &mut self,
        _target: &mut Self,
        _state: &mut Self::State,
        _world: &mut World,
        _children: &mut Vec<Entity>,
    ) {
    }
}

impl Compose for &'static str {
    type State = Entity;

    fn build(&mut self, world: &mut World, children: &mut Vec<Entity>) -> Self::State {
        let entity = world.spawn(TextBundle::from_section(
            self.to_owned(),
            Default::default(),
        ));
        let id = entity.id();
        children.push(id);
        id
    }

    fn rebuild(
        &mut self,
        target: &mut Self,
        state: &mut Self::State,
        world: &mut World,
        children: &mut Vec<Entity>,
    ) {
        children.push(*state);

        if self != target {
            world.get_mut::<Text>(*state).unwrap().sections[0] =
                TextSection::new(self.to_owned(), TextStyle::default());
        }
    }
}

impl Compose for String {
    type State = Entity;

    fn build(&mut self, world: &mut World, children: &mut Vec<Entity>) -> Self::State {
        let entity = world.spawn(TextBundle::from_section(self.clone(), Default::default()));
        let id = entity.id();
        children.push(id);
        id
    }

    fn rebuild(
        &mut self,
        target: &mut Self,
        state: &mut Self::State,
        world: &mut World,
        children: &mut Vec<Entity>,
    ) {
        children.push(*state);

        if self != target {
            world.get_mut::<Text>(*state).unwrap().sections[0] =
                TextSection::new(self.clone(), TextStyle::default());
        }
    }
}

impl<C1: Compose, C2: Compose, C3: Compose> Compose for (C1, C2, C3) {
    type State = (C1::State, C2::State, C3::State);

    fn build(&mut self, world: &mut World, children: &mut Vec<Entity>) -> Self::State {
        (
            self.0.build(world, children),
            self.1.build(world, children),
            self.2.build(world, children),
        )
    }

    fn rebuild(
        &mut self,
        target: &mut Self,
        state: &mut Self::State,
        world: &mut World,
        children: &mut Vec<Entity>,
    ) {
        self.0.rebuild(&mut target.0, &mut state.0, world, children);
        self.1.rebuild(&mut target.1, &mut state.1, world, children);
        self.2.rebuild(&mut target.2, &mut state.2, world, children);
    }
}

pub fn button<C: Compose>(content: C) -> Button<C> {
    Button {
        content,
        on_click: None,
    }
}

pub struct Button<C> {
    content: C,
    on_click: Option<Box<dyn FnMut(&mut World) + Send + Sync>>,
}

impl<C> Button<C> {
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

impl<C: Compose> Compose for Button<C> {
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
                entity: id,
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
    entity: Entity,
    handler: Option<Box<dyn FnMut(&mut World) + Send + Sync>>,
}

pub fn handler_system(world: &mut World) {
    let mut query = world.query_filtered::<
        (&Interaction, &mut ClickHandler),
        (Changed<Interaction>, With<bevy::ui::widget::Button>),
    >();

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

pub trait AnyCompose: Send + Sync {
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn build_any(
        &mut self,
        world: &mut World,
        children: &mut Vec<Entity>,
    ) -> Box<dyn Any + Send + Sync>;

    fn rebuild_any(
        &mut self,
        target: &mut dyn Any,
        state: &mut dyn Any,
        world: &mut World,
        children: &mut Vec<Entity>,
    );
}

impl<C: Compose> AnyCompose for C {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn build_any(
        &mut self,
        world: &mut World,
        children: &mut Vec<Entity>,
    ) -> Box<dyn Any + Send + Sync> {
        Box::new(self.build(world, children))
    }

    fn rebuild_any(
        &mut self,
        target: &mut dyn Any,
        state: &mut dyn Any,
        world: &mut World,
        children: &mut Vec<Entity>,
    ) {
        self.rebuild(
            target.downcast_mut().unwrap(),
            state.downcast_mut().unwrap(),
            world,
            children,
        )
    }
}

#[derive(Component)]
pub struct Composer {
    compose: Option<Box<dyn FnMut(&mut World) -> Box<dyn AnyCompose> + Send + Sync>>,
    state: Option<(Box<dyn AnyCompose>, Box<dyn Any + Send + Sync>)>,
}

impl Composer {
    pub fn new<Marker, C: Compose>(compose_fn: impl IntoSystem<(), C, Marker>) -> Self {
        let mut system_cell = Some(IntoSystem::<(), C, Marker>::into_system(compose_fn));
        let mut id_cell = None;
        Self {
            compose: Some(Box::new(move |world| {
                if let Some(system) = system_cell.take() {
                    let id = world.register_system(system);
                    id_cell = Some(id);
                }

                let id = id_cell.unwrap();
                Box::new(world.run_system(id).unwrap())
            })),
            state: None,
        }
    }
}

pub fn compose(world: &mut World) {
    let mut query = world.query::<&mut Composer>();
    let mut composers = query
        .iter_mut(world)
        .map(|mut composer| (composer.compose.take(), composer.state.take()))
        .collect::<Vec<_>>();

    for (compose_fn, state) in &mut composers {
        let mut compose = compose_fn.as_mut().unwrap()(world);
        let mut children = Vec::new();

        if let Some((target, state)) = state {
            compose.rebuild_any(target.as_any_mut(), &mut **state, world, &mut children)
        } else {
            let s = compose.build_any(world, &mut children);
            *state = Some((compose, s));

            world
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    background_color: BackgroundColor(Color::BLACK),
                    ..Default::default()
                })
                .push_children(&children);
        }
    }

    for (idx, mut composer) in query.iter_mut(world).enumerate() {
        composer.compose = composers[idx].0.take();
        composer.state = composers[idx].1.take();
    }
}
