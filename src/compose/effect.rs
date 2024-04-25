use crate::Compose;
use bevy::{
    app::App,
    ecs::{entity::Entity, system::{ParamSet, SystemParam, SystemParamFunction}},
};
use std::marker::PhantomData;

pub fn effect<D, F, Marker>(deps: D, f: F) -> Effect<D, F, Marker> {
    Effect {
        deps: Some(deps),
        f,
        _marker: PhantomData,
    }
}

pub struct Effect<D, F, Marker> {
    deps: Option<D>,
    f: F,
    _marker: PhantomData<Marker>,
}

impl<D, F, Marker> Compose for Effect<D, F, Marker>
where
    D: PartialEq + Send + Sync + 'static,
    F: SystemParamFunction<Marker, In = (), Out = ()>,
{
    type State = Option<D>;

    type Input<'w, 's> = ParamSet<'w, 's, (F::Param,)>;

    fn setup(_app: &mut App, _parent: Option<Entity>) -> Self::State {
        None
    }

    fn run(
        mut self,
        state: &mut Self::State,
        mut input: <Self::Input<'_, '_> as SystemParam>::Item<'_, '_>,
    ) {
        let deps = self.deps.take().unwrap();

        if let Some(last) = state {
            if deps != *last {
                self.f.run((), input.p0());
                *state = Some(deps);
            }
        } else {
            self.f.run((), input.p0());
            *state = Some(deps);
        }
    }
}
