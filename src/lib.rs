use bevy::{
    ecs::{component::Component, entity::Entity, world::World},
    text::{Text, TextSection, TextStyle},
    ui::node_bundles::TextBundle,
};
use std::any::Any;

pub trait Compose: Send + Sync + 'static {
    type State: Send + Sync + 'static;

    fn build(&mut self, world: &mut World) -> Self::State;

    fn rebuild(&mut self, target: &mut Self, state: &mut Self::State, world: &mut World);
}

impl Compose for () {
    type State = ();

    fn build(&mut self, world: &mut World) -> Self::State {}

    fn rebuild(&mut self, target: &mut Self, state: &mut Self::State, world: &mut World) {}
}

impl Compose for String {
    type State = Entity;

    fn build(&mut self, world: &mut World) -> Self::State {
        let entity = world.spawn(TextBundle::from_section(self.clone(), Default::default()));
        entity.id()
    }

    fn rebuild(&mut self, target: &mut Self, state: &mut Self::State, world: &mut World) {
        if self != target {
            world.get_mut::<Text>(*state).unwrap().sections[0] =
                TextSection::new(self.clone(), TextStyle::default());
        }
    }
}

impl<C1: Compose, C2: Compose> Compose for (C1, C2 ){
    type State = (C1::State, C2::State);

    fn build(&mut self, world: &mut World) -> Self::State {
        (self.0.build(world), self.1.build(world))
    }

    fn rebuild(&mut self, target: &mut Self, state: &mut Self::State, world: &mut World) {
        self.0.rebuild(&mut target.0, &mut state.0, world);
        self.1.rebuild(&mut target.1, &mut state.1, world);
    }
} 

pub trait AnyCompose: Send + Sync {
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn build_any(&mut self, world: &mut World) -> Box<dyn Any + Send + Sync>;

    fn rebuild_any(&mut self, target: &mut dyn Any, state: &mut dyn Any, world: &mut World);
}

impl<C: Compose> AnyCompose for C {
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn build_any(&mut self, world: &mut World) -> Box<dyn Any + Send + Sync> {
        Box::new(self.build(world))
    }

    fn rebuild_any(&mut self, target: &mut dyn Any, state: &mut dyn Any, world: &mut World) {
        self.rebuild(
            target.downcast_mut().unwrap(),
            state.downcast_mut().unwrap(),
            world,
        )
    }
}

#[derive(Component)]
pub struct Composer {
    compose: Option<Box<dyn FnMut(&mut World) -> Box<dyn AnyCompose> + Send + Sync>>,
    state: Option<(Box<dyn AnyCompose>, Box<dyn Any + Send + Sync>)>,
}

impl Composer {
    pub fn new<C: Compose>(mut compose_fn: impl FnMut(&mut World) -> C + Send + Sync + 'static) -> Self {
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

        if let Some((target, state)) = state {
            compose.rebuild_any(target.as_any_mut(), &mut **state, world)
        } else {
            let s = compose.build_any(world);
            *state = Some((compose, s));
        }
    }

    for (idx, mut composer) in query.iter_mut(world).enumerate() {
        composer.compose = composers[idx].0.take();
        composer.state = composers[idx].1.take();
    }
}
