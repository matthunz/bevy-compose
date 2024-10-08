use bevy::prelude::*;
use bevy_compose::TemplatePlugin;

#[derive(Clone, Copy, PartialEq, Component, Deref)]
struct Health(i32);

#[derive(Component, Deref)]
struct Damage(i32);

#[derive(Component)]
struct Zombie;

fn main() {
    App::new()
        .add_plugins(TemplatePlugin::default().with_template(
            Zombie,
            (
                || Health(100),
                |entity: In<Entity>, health_query: Query<&Health>| {
                    let health = health_query.get(*entity).unwrap();
                    Damage(**health * 2)
                },
            ),
        ))
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
    }
}
