use bevy::prelude::*;
use bevy_compose::TemplatePlugin;

#[derive(Component)]
struct Health(i32);

#[derive(Component)]
struct Zombie;

fn main() {
    App::new()
        .add_plugins(TemplatePlugin::default().with_template(Zombie, || Health(100)))
        .add_systems(Startup, setup)
        .add_systems(PostUpdate, debug)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Zombie);
}

fn debug(query: Query<&Health>) {
    for health in &query {
        dbg!(health.0);
    }
}
