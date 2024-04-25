use crate::Compose;
use bevy::{
    ecs::{
        entity::Entity,
        system::{Commands, ParamSet},
    },
    hierarchy::BuildWorldChildren,
    ui::{node_bundles::NodeBundle, Style},
};

pub fn flex<C: Compose>(content: C) -> Flex<C> {
    Flex { content }
}

pub struct Flex<C> {
    content: C,
}

impl<C: Compose> Compose for Flex<C> {
    type State = (Entity, bool, C::State);

    type Input<'w, 's> = (Commands<'w, 's>, ParamSet<'w, 's, (C::Input<'w, 's>,)>);

    fn setup(app: &mut bevy::prelude::App, parent: Option<Entity>) -> Self::State {
        let mut entity = app.world.spawn(NodeBundle::default());
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
