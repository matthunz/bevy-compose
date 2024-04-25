use crate::Compose;
use bevy::{
    app::Update,
    ecs::{
        component::{Component, SparseStorage},
        entity::Entity,
        query::Changed,
        system::{Commands, ParamSet, Query, SystemParamFunction},
    },
    hierarchy::BuildWorldChildren,
    ui::{node_bundles::NodeBundle, Interaction, Style},
};
use std::marker::PhantomData;

pub fn flex<C: Compose>(content: C) -> Flex<C> {
    Flex { content }
}

pub struct Flex<C> {
    content: C,
}

impl<C> Flex<C> {
    pub fn on_click<F, Marker>(self, f: F) -> OnClick<C, F, Marker>
    where
        F: SystemParamFunction<Marker, In = (), Out = ()>,
    {
        OnClick {
            flex: self,
            f,
            _marker: PhantomData,
        }
    }
}

impl<C: Compose> Compose for Flex<C> {
    type State = (Entity, bool, C::State);

    type Input<'w, 's> = (Commands<'w, 's>, ParamSet<'w, 's, (C::Input<'w, 's>,)>);

    fn setup(app: &mut bevy::prelude::App, parent: Option<Entity>) -> Self::State {
        let mut entity = app
            .world
            .spawn((NodeBundle::default(), Interaction::default()));
        if let Some(parent) = parent {
            entity.set_parent(parent);
        }
        let id = entity.id();

        let content_state = C::setup(app, Some(id));
        (id, false, content_state)
    }

    fn run(
        self,
        (entity, is_init, content_state): &mut Self::State,
        (mut commands, mut params): <Self::Input<'_, '_> as bevy::ecs::system::SystemParam>::Item<
            '_,
            '_,
        >,
    ) {
        if !*is_init {
            commands.entity(*entity).insert(NodeBundle {
                style: Style {
                    ..Default::default()
                },
                ..Default::default()
            });
            *is_init = true;
        }

        self.content.run(content_state, params.p0());
    }
}

pub struct ClickHandler<F>(Option<F>);

impl<F: Send + Sync + 'static> Component for ClickHandler<F> {
    type Storage = SparseStorage;
}

pub struct OnClick<C, F, Marker> {
    flex: Flex<C>,
    f: F,
    _marker: PhantomData<Marker>,
}

impl<C, F, Marker> Compose for OnClick<C, F, Marker>
where
    C: Compose,
    F: SystemParamFunction<Marker, In = (), Out = ()>,
    F::Param: 'static,
{
    type State = <Flex<C> as Compose>::State;

    type Input<'w, 's> = (
        <Flex<C> as Compose>::Input<'w, 's>,
        Query<'w, 's, &'static mut ClickHandler<F>>,
    );

    fn setup(app: &mut bevy::prelude::App, parent: Option<Entity>) -> Self::State {
        let state = Flex::<C>::setup(app, parent);
        let entity = state.0;

        app.world.entity_mut(entity).insert(ClickHandler::<F>(None));

        app.add_systems(
            Update,
            move |mut interaction_query: Query<
                (&Interaction, &mut ClickHandler<F>),
                Changed<Interaction>,
            >,
                  mut params: ParamSet<(F::Param,)>| {
                if let Ok((Interaction::Pressed, mut handler)) = interaction_query.get_mut(entity) {
                    if let Some(ref mut f) = handler.0 {
                        f.run((), params.p0());
                    }
                }
            },
        );

        state
    }

    fn run(
        self,
        state: &mut Self::State,
        (input, mut query): <Self::Input<'_, '_> as bevy::ecs::system::SystemParam>::Item<'_, '_>,
    ) {
        let mut handler = query.get_mut(state.0).unwrap();
        handler.0 = Some(self.f);

        self.flex.run(state, input);
    }
}
