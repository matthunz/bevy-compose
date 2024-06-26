use crate::Compose;
use bevy::{
    app::{App, Update},
    ecs::{
        component::{Component, SparseStorage},
        entity::Entity,
        system::{ParamSet, Query, SystemParam, SystemParamFunction},
    },
};
use std::marker::PhantomData;

pub fn lazy<Marker, F, C>(f: F) -> Lazy<F, (Marker, C)>
where
    Marker: 'static,
    F: SystemParamFunction<Marker, In = (), Out = C>,
    F::Param: 'static,
    C: Compose + 'static,
{
    Lazy {
        f: Some(f),
        _marker: PhantomData,
    }
}

pub struct Lazy<F, C> {
    f: Option<F>,
    _marker: PhantomData<C>,
}

impl<Marker, F, C> Compose for Lazy<F, (Marker, C)>
where
    Marker: 'static,
    F: SystemParamFunction<Marker, In = (), Out = C>,
    F::Param: 'static,
    C: Compose + 'static,
{
    type State = Entity;
    type Input<'w, 's> = Query<'w, 's, &'static mut LazyFunction<F>>;

    fn setup(app: &mut App, parent: Option<Entity>) -> Self::State {
        let entity = app.world.spawn(LazyFunction::<F> { f: None }).id();
        let content_state = C::setup(app, parent);
        app.add_systems(
            Update,
            make_lazy_system::<Marker, F, C>(entity, content_state),
        );

        entity
    }

    fn run(
        mut self,
        state: &mut Self::State,
        mut input: <Self::Input<'_, '_> as SystemParam>::Item<'_, '_>,
    ) {
        if let Some(f) = self.f.take() {
            let mut x = input.get_mut(*state).unwrap();
            x.f = Some(f);
        }
    }
}

pub struct LazyFunction<F> {
    f: Option<F>,
}

impl<F> Component for LazyFunction<F>
where
    F: Send + Sync + 'static,
{
    type Storage = SparseStorage;
}

fn make_lazy_system<Marker, F, C>(
    entity: Entity,
    mut state: C::State,
) -> impl FnMut(ParamSet<(F::Param, C::Input<'_, '_>)>, Query<&mut LazyFunction<F>>)
where
    F: SystemParamFunction<Marker, In = (), Out = C>,
    C: Compose,
{
    move |mut p, mut query| {
        let mut wrapper = query.get_mut(entity).unwrap();
        if let Some(f) = &mut wrapper.f {
            let content = f.run((), p.p0());
            content.run(&mut state, p.p1());
        }
    }
}
