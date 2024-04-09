use bevy::{
    ecs::{component::Component, entity::Entity, world::World},
    hierarchy::BuildWorldChildren,
    render::color::Color,
    text::{Text, TextSection, TextStyle},
    ui::{
        node_bundles::{ButtonBundle, TextBundle},
        AlignItems, BackgroundColor, JustifyContent, Style, UiRect, Val,
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

    fn build(&mut self, world: &mut World, children: &mut Vec<Entity>) -> Self::State {}

    fn rebuild(
        &mut self,
        target: &mut Self,
        state: &mut Self::State,
        world: &mut World,
        children: &mut Vec<Entity>,
    ) {
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

impl<C1: Compose, C2: Compose> Compose for (C1, C2) {
    type State = (C1::State, C2::State);

    fn build(&mut self, world: &mut World, children: &mut Vec<Entity>) -> Self::State {
        (self.0.build(world, children), self.1.build(world, children))
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
    }
}

pub fn button<C: Compose>(content: C) -> Button<C> {
    Button { content }
}

pub struct Button<C> {
    content: C,
}

impl<C: Compose> Compose for Button<C> {
    type State = Entity;

    fn build(&mut self, world: &mut World, children: &mut Vec<Entity>) -> Self::State {
        let parent_children = mem::take(children);
        self.content.build(world, children);
        let children = mem::replace(children, parent_children);

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
        entity.push_children(&children);
        entity.id()
    }

    fn rebuild(
        &mut self,
        target: &mut Self,
        state: &mut Self::State,
        world: &mut World,
        children: &mut Vec<Entity>,
    ) {
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
    pub fn new<C: Compose>(
        mut compose_fn: impl FnMut(&mut World) -> C + Send + Sync + 'static,
    ) -> Self {
        Self {
            compose: Some(Box::new(move |world| Box::new(compose_fn(world)))),
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
        }
    }

    for (idx, mut composer) in query.iter_mut(world).enumerate() {
        composer.compose = composers[idx].0.take();
        composer.state = composers[idx].1.take();
    }
}
