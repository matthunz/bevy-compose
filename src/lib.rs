use bevy::{
    app::{App, Startup, Update},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, EntityCommands, Local, ParamSet, Query, SystemParam},
        world::Mut,
    },
    ui::IsDefaultUiCamera,
    DefaultPlugins,
};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

pub mod compose;
pub use compose::Compose;

pub fn run<C>(mut compose_fn: impl FnMut() -> C + Send + Sync + 'static)
where
    C: Compose + Send + Sync + 'static,
{
    let mut app = App::new();
    let mut state = C::setup(&mut app, None);
    app.add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, move |mut params: ParamSet<(C::Input<'_, '_>,)>| {
            let compose = compose_fn();
            compose.run(&mut state, params.p0());
        })
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), IsDefaultUiCamera));
}

#[derive(SystemParam)]
pub struct UseState<'w, 's, T: Component> {
    commands: Commands<'w, 's>,
    cell: Local<'s, Option<Entity>>,
    query: Query<'w, 's, &'static mut T>,
    _marker: PhantomData<T>,
}

impl<T> UseState<'_, '_, T>
where
    T: Component,
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

pub enum StateHandle<'a, T: Component> {
    Borrowed(Mut<'a, T>),
    Owned {
        value_cell: Option<T>,
        entity_commands: EntityCommands<'a>,
    },
}

impl<'a, T: Component> Deref for StateHandle<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            StateHandle::Borrowed(value) => value,
            StateHandle::Owned {
                value_cell: value,
                entity_commands: _,
            } => value.as_ref().unwrap(),
        }
    }
}

impl<'a, T: Component> DerefMut for StateHandle<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            StateHandle::Borrowed(value) => &mut *value,
            StateHandle::Owned {
                value_cell,
                entity_commands: _,
            } => value_cell.as_mut().unwrap(),
        }
    }
}

impl<'a, T: Component> Drop for StateHandle<'a, T> {
    fn drop(&mut self) {
        match self {
            StateHandle::Borrowed(_) => {}
            StateHandle::Owned {
                value_cell: value,
                entity_commands,
            } => {
                entity_commands.insert(value.take().unwrap());
            }
        }
    }
}
