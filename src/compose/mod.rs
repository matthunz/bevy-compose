use bevy::{
    app::{App, Update},
    ecs::{
        component::{Component, SparseStorage},
        entity::Entity,
        system::{ParamSet, Query, SystemParam},
    },
};

mod effect;
pub use self::effect::{effect, Effect};

mod lazy;
pub use self::lazy::{lazy, Lazy, LazyFunction};

pub trait Compose {
    type State: Send + Sync + 'static;
    type Input<'w, 's>: SystemParam;

    fn setup(app: &mut App) -> Self::State;

    fn run(
        self,
        state: &mut Self::State,
        input: <Self::Input<'_, '_> as SystemParam>::Item<'_, '_>,
    );
}

impl Compose for () {
    type State = ();

    type Input<'w, 's> = ();

    fn setup(_app: &mut App) -> Self::State {}

    fn run(self, _state: &mut Self::State, _input: Self::Input<'_, '_>) {}
}

pub struct TupleCompose<C>(Option<C>);

impl<C: Send + Sync + 'static> Component for TupleCompose<C> {
    type Storage = SparseStorage;
}

impl<C1, C2> Compose for (C1, C2)
where
    C1: Compose + Send + Sync + 'static,
    C2: Compose + Send + Sync + 'static,
{
    type State = [Entity; 2];

    type Input<'w, 's> = (
        Query<'w, 's, &'static mut TupleCompose<C1>>,
        Query<'w, 's, &'static mut TupleCompose<C2>>,
    );

    fn setup(app: &mut App) -> Self::State {
        let c1 = app.world.spawn(TupleCompose::<C1>(None)).id();
        let mut c1_state = C1::setup(app);

        let c2 = app.world.spawn(TupleCompose::<C2>(None)).id();
        let mut c2_state = C2::setup(app);

        app.add_systems(
            Update,
            move |mut q: Query<&mut TupleCompose<C1>>,
                  mut params: ParamSet<(C1::Input<'_, '_>,)>| {
                let mut content = q.get_mut(c1).unwrap();
                if let Some(content) = content.0.take() {
                    content.run(&mut c1_state, params.p0());
                }
            },
        );

        app.add_systems(
            Update,
            move |mut q: Query<&mut TupleCompose<C2>>,
                  mut params: ParamSet<(C2::Input<'_, '_>,)>| {
                let mut content = q.get_mut(c2).unwrap();
                if let Some(content) = content.0.take() {
                    content.run(&mut c2_state, params.p0());
                }
            },
        );

        [c1, c2]
    }

    fn run(
        self,
        [entity1, entity2]: &mut Self::State,
        (mut query1, mut query2): <Self::Input<'_, '_> as SystemParam>::Item<'_, '_>,
    ) {
        let mut c1 = query1.get_mut(*entity1).unwrap();
        c1.0 = Some(self.0);

        let mut c2 = query2.get_mut(*entity2).unwrap();
        c2.0 = Some(self.1);
    }
}
