use crate::{AnyCompose, Compose};
use bevy::prelude::*;
use std::any::Any;

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
            let _target = target.unwrap();

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
