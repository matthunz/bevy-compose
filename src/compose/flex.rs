use crate::Compose;
use bevy::{
    ecs::{
        entity::Entity,
        system::{Commands, ParamSet},
    },
    ui::node_bundles::NodeBundle,
};

pub fn flex<C: Compose>(content: C) -> Flex<C> {
    Flex { content }
}

pub struct Flex<C> {
    content: C,
}

impl<C: Compose> Compose for Flex<C> {
    type State = (Option<Entity>, C::State);

    type Input<'w, 's> = (Commands<'w, 's>, ParamSet<'w, 's, (C::Input<'w, 's>,)>);

    fn setup(app: &mut bevy::prelude::App) -> Self::State {
        let content_state = C::setup(app);

        (None, content_state)
    }

    fn run(
        self,
        (entity_cell, content_state): &mut Self::State,
        (mut commands, mut params): <Self::Input<'_, '_> as bevy::ecs::system::SystemParam>::Item<
            '_,
            '_,
        >,
    ) {
        if let Some(_entity) = entity_cell {
        } else {
            let entity = commands.spawn(NodeBundle::default()).id();
            *entity_cell = Some(entity);
        }

        self.content.run(content_state, params.p0());
    }
}
