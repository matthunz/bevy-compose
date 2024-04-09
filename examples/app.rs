use bevy::prelude::*;
use bevy_compose::{compose, Compose, Composer};

#[derive(Resource)]
struct Count(i32);

fn ui() -> impl Compose {
    format!("Hello World!")
}

fn main() {
    let mut app = App::new();

    app.world.insert_resource(Count(0));
    app.world.spawn(Composer::new(ui));
    app.world.spawn((Camera2dBundle::default(), IsDefaultUiCamera));

    app.add_plugins(DefaultPlugins)
        .add_systems(Update, compose)
        .run();
}
