use bevy::{
    app::{Plugin, Update},
    ecs::{component::Component, entity::Entity, system::IntoSystem, world::World},
    hierarchy::BuildWorldChildren,
    render::color::Color,
    ui::{node_bundles::NodeBundle, AlignItems, BackgroundColor, FlexDirection, Style, Val},
};
use compose::handler_system;
use std::{any::Any, sync::Arc};

pub mod compose;
pub use compose::Compose;

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

pub fn lazy<C: Compose, Marker>(system: impl IntoSystem<(), C, Marker>) -> Lazy {
    let mut cell = Some(IntoSystem::<(), C, Marker>::into_system(system));

    let system: Option<
        Box<
            dyn FnMut(Option<&mut dyn Any>, &mut World, &mut Option<LazyState>, &mut Vec<Entity>)
                + Send
                + Sync,
        >,
    > = Some(Box::new(move |target, world, state_cell, children| {
        if let Some(ref mut state) = state_cell {
            let target = target.unwrap();

            let mut compose = (state.system)(world);
            compose.rebuild_any(
                state.compose.as_any_mut(),
                &mut *state.state,
                world,
                children,
            );
        } else {
            let system = cell.take().unwrap();
            let system_id = world.register_system(system);

            let mut compose = world.run_system(system_id).unwrap();
            let state = compose.build_any(world, children);

            *state_cell = Some(LazyState {
                system: Box::new(move |world| {
                    let compose = world.run_system(system_id).unwrap();
                    Box::new(compose)
                }),
                compose: Box::new(compose),
                state,
            })
        }
    }));

    Lazy { system }
}

pub struct LazyState {
    system: Box<dyn FnMut(&mut World) -> Box<dyn AnyCompose> + Send + Sync>,
    compose: Box<dyn AnyCompose>,
    state: Box<dyn Any + Send + Sync>,
}

pub struct Lazy {
    system: Option<
        Box<
            dyn FnMut(Option<&mut dyn Any>, &mut World, &mut Option<LazyState>, &mut Vec<Entity>)
                + Send
                + Sync,
        >,
    >,
}

impl Compose for Lazy {
    type State = Option<LazyState>;

    fn build(&mut self, world: &mut World, children: &mut Vec<Entity>) -> Self::State {
        let mut state_cell = None;
        self.system.as_mut().unwrap()(None, world, &mut state_cell, children);
        state_cell
    }

    fn rebuild(
        &mut self,
        target: &mut Self,
        state: &mut Self::State,
        world: &mut World,
        children: &mut Vec<Entity>,
    ) {
        self.system.as_mut().unwrap()(Some(target), world, state, children);
    }
}

pub struct ComposePlugin {
    make_composer: Arc<dyn Fn() -> Composer + Send + Sync>,
}

impl ComposePlugin {
    pub fn new<C: Compose, Marker>(
        compose_fn: impl IntoSystem<(), C, Marker> + Clone + Send + Sync + 'static,
    ) -> Self {
        Self {
            make_composer: Arc::new(move || Composer::new(compose_fn.clone())),
        }
    }
}

impl Plugin for ComposePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.world.spawn((self.make_composer)());

        app.add_systems(Update, (compose, handler_system));
    }
}
