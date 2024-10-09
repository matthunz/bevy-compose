use bevy::prelude::*;
use bevy_compose::{Template, TemplatePlugin};

#[derive(Clone, Copy, PartialEq, Component, Deref)]
struct Health(i32);

#[derive(Component, Deref)]
struct Damage(i32);

#[derive(Component)]
struct Zombie;

fn main() {
    App::new()
        .add_plugins(TemplatePlugin::default().add(Template::new(
            // Spawning a Zombie will spawn the following components:
            Zombie,
            (
                // This only runs once.
                || Health(100),
                // This runs every time a `Health` component is updated,
                // and it's guraranteed to run before other systems using the `Damage` component.
                |entity: In<Entity>, health_query: Query<&Health>| {
                    let health = health_query.get(*entity).unwrap();
                    Damage(**health * 2)
                },
            ),
        )))
        .add_systems(Startup, setup)
        .add_systems(PostUpdate, debug)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Zombie);
}

fn debug(query: Query<&Damage>) {
    for dmg in &query {
        dbg!(**dmg);
        // 200.
    }
}
