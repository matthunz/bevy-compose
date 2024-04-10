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

type AnyComposeFn = Box<dyn FnMut(&mut World) -> Box<dyn AnyCompose> + Send + Sync>;

#[derive(Component)]
pub struct Composer {
    compose: Option<AnyComposeFn>,
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
