use bevy::{
    app::{App, Update}, ecs::{
        component::{Component, SparseStorage},
        entity::Entity,
        system::{
            Commands, EntityCommands, Local, ParamSet, Query, SystemParam, SystemParamFunction,
        },
        world::Mut,
    }, prelude::{Deref, DerefMut}, DefaultPlugins
};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

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

    fn setup(app: &mut App) -> Self::State {
        let entity = app.world.spawn(LazyFunction::<F> { f: None }).id();
        let content_state = C::setup(app);
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

    fn setup(_app: &mut App) -> Self::State {
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

pub fn run<C>(mut compose_fn: impl FnMut() -> C + Send + Sync + 'static)
where
    C: Compose + Send + Sync + 'static,
{
    let mut app = App::new();
    let mut state = C::setup(&mut app);
    app.add_systems(Update, move |mut params: ParamSet<(C::Input<'_, '_>,)>| {
        let compose = compose_fn();
        compose.run(&mut state, params.p0());
    });

    app.add_plugins(DefaultPlugins);
    app.run();
}

#[derive(Deref, DerefMut)]
pub struct StateComponent<T>(pub T);

impl<T: Send + Sync + 'static> Component for StateComponent<T> {
    type Storage = SparseStorage;
}

#[derive(SystemParam)]
pub struct UseState<'w, 's, T: Send + Sync + 'static> {
    commands: Commands<'w, 's>,
    cell: Local<'s, Option<Entity>>,
    query: Query<'w, 's, &'static mut StateComponent<T>>,
    _marker: PhantomData<T>,
}

impl<T> UseState<'_, '_, T>
where
    T: Send + Sync + 'static,
{
    pub fn use_state(&mut self, make_value: impl FnOnce() -> T) -> (StateHandle<T>, Entity) {
        if let Some(entity) = *self.cell {
            let state = self.query.get_mut(entity).unwrap();
            (StateHandle::Borrowed(state), entity)
        } else {
            let entity_commands = self.commands.spawn_empty();
            *self.cell = Some(entity_commands.id());

            let entity = entity_commands.id();
            (
                StateHandle::Owned {
                    value_cell: Some(make_value()),
                    entity_commands,
                },
                entity,
            )
        }
    }
}

pub enum StateHandle<'a, T: Send + Sync + 'static> {
    Borrowed(Mut<'a, StateComponent<T>>),
    Owned {
        value_cell: Option<T>,
        entity_commands: EntityCommands<'a>,
    },
}

impl<'a, T: Send + Sync + 'static> Deref for StateHandle<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            StateHandle::Borrowed(value) => &value.0,
            StateHandle::Owned {
                value_cell: value,
                entity_commands: _,
            } => value.as_ref().unwrap(),
        }
    }
}

impl<'a, T: Send + Sync + 'static> DerefMut for StateHandle<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            StateHandle::Borrowed(value) => &mut value.0,
            StateHandle::Owned {
                value_cell,
                entity_commands: _,
            } => value_cell.as_mut().unwrap(),
        }
    }
}

impl<'a, T: Send + Sync + 'static> Drop for StateHandle<'a, T> {
    fn drop(&mut self) {
        match self {
            StateHandle::Borrowed(_) => {}
            StateHandle::Owned {
                value_cell: value,
                entity_commands,
            } => {
                entity_commands.insert(StateComponent(value.take().unwrap()));
            }
        }
    }
}
