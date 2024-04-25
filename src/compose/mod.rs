use bevy::{
    app::{App, Update},
    ecs::{
        component::{Component, SparseStorage},
        entity::Entity,
        system::{Commands, ParamSet, Query, SystemParam},
    },
    hierarchy::BuildWorldChildren,
    text::Text,
    ui::node_bundles::TextBundle,
};

mod effect;
pub use self::effect::{effect, Effect};

mod flex;
pub use self::flex::{flex, Flex};

mod lazy;
pub use self::lazy::{lazy, Lazy, LazyFunction};

pub trait Compose {
    type State: Send + Sync + 'static;
    type Input<'w, 's>: SystemParam;

    fn setup(app: &mut App, parent: Option<Entity>) -> Self::State;

    fn run(
        self,
        state: &mut Self::State,
        input: <Self::Input<'_, '_> as SystemParam>::Item<'_, '_>,
    );
}

impl Compose for () {
    type State = ();

    type Input<'w, 's> = ();

    fn setup(_app: &mut App, _parent: Option<Entity>) -> Self::State {}

    fn run(self, _state: &mut Self::State, _input: Self::Input<'_, '_>) {}
}

impl Compose for &'static str {
    type State = (Option<Self>, Entity);

    type Input<'w, 's> = (Commands<'w, 's>, Query<'w, 's, &'static mut Text>);

    fn setup(app: &mut App, parent: Option<Entity>) -> Self::State {
        let mut entity = app.world.spawn(TextBundle::default());
        if let Some(parent) = parent {
            entity.set_parent(parent);
        }

        (None, entity.id())
    }

    fn run(
        self,
        (last_cell, entity): &mut Self::State,
        (mut commands, mut text_query): <Self::Input<'_, '_> as SystemParam>::Item<'_, '_>,
    ) {
        if let Some(last) = last_cell {
            if self != *last {
                text_query.get_mut(*entity).unwrap().sections[0].value = self.to_owned();
                *last_cell = Some(self)
            }
        } else {
            commands
                .entity(*entity)
                .insert(TextBundle::from_section(self, Default::default()));
            *last_cell = Some(self)
        }
    }
}

impl Compose for String {
    type State = (Option<Self>, Entity);

    type Input<'w, 's> = (Commands<'w, 's>, Query<'w, 's, &'static mut Text>);

    fn setup(app: &mut App, parent: Option<Entity>) -> Self::State {
        let mut entity = app.world.spawn(TextBundle::default());
        if let Some(parent) = parent {
            entity.set_parent(parent);
        }

        (None, entity.id())
    }

    fn run(
        self,
        (last_cell, entity): &mut Self::State,
        (mut commands, mut text_query): <Self::Input<'_, '_> as SystemParam>::Item<'_, '_>,
    ) {
        if let Some(last) = last_cell {
            if self != *last {
                text_query.get_mut(*entity).unwrap().sections[0].value = self.to_owned();
                *last_cell = Some(self)
            }
        } else {
            commands
                .entity(*entity)
                .insert(TextBundle::from_section(self.clone(), Default::default()));
            *last_cell = Some(self)
        }
    }
}

pub struct TupleCompose<C>(Option<C>);

impl<C: Send + Sync + 'static> Component for TupleCompose<C> {
    type Storage = SparseStorage;
}

macro_rules! impl_compose_for_tuple {
    (($($t:tt: $idx:tt),*), $len:tt) => {
        #[allow(non_snake_case)]
        impl<$($t: Compose + Send + Sync + 'static),*> Compose for ($($t),*) {
            type State = [Entity; $len];

            type Input<'w, 's> = ($(Query<'w, 's, &'static mut TupleCompose<$t>>),*);

            fn setup(app: &mut App, parent: Option<Entity>) -> Self::State {
                $(
                    let $t = app.world.spawn(TupleCompose::<$t>(None)).id();
                    let mut c_state = $t::setup(app, parent);
                    app.add_systems(
                        Update,
                        move |mut q: Query<&mut TupleCompose<$t>>,
                              mut params: ParamSet<($t::Input<'_, '_>,)>| {
                            let mut content = q.get_mut($t).unwrap();
                            if let Some(content) = content.0.take() {
                                content.run(&mut c_state, params.p0());
                            }
                        },
                    );
                )*

                [$($t),*]
            }

            fn run(
                self,
                state: &mut Self::State,
                mut input: <Self::Input<'_, '_> as SystemParam>::Item<'_, '_>,
            ) {
                $(
                    let mut c = input.$idx.get_mut(state[$idx]).unwrap();
                    c.0 = Some(self.$idx);
                )*
            }
        }
    };
}

impl_compose_for_tuple!((C1: 0, C2: 1), 2);
impl_compose_for_tuple!((C1: 0, C2: 1, C3: 2), 3);
impl_compose_for_tuple!((C1: 0, C2: 1, C3: 2, C4: 3), 4);
impl_compose_for_tuple!((C1: 0, C2: 1, C3: 2, C4: 3, C5: 4), 5);
impl_compose_for_tuple!((C1: 0, C2: 1, C3: 2, C4: 3, C5: 4, C6: 5), 6);
impl_compose_for_tuple!((C1: 0, C2: 1, C3: 2, C4: 3, C5: 4, C6: 5, C7: 6), 7);
impl_compose_for_tuple!((C1: 0, C2: 1, C3: 2, C4: 3, C5: 4, C6: 5, C7: 6, C8: 7), 8);
