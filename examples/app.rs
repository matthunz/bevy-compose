use bevy::prelude::*;
use bevy_compose::{composer, Compose};

#[derive(Resource)]
struct Count(i32);

fn ui(count_query: Res<Count>) -> impl Compose {
    format!("{}", count_query.0)
}

fn main() {
    let mut app = App::new();

    app.world.insert_resource(Count(0));

    app.add_plugins(DefaultPlugins)
        .add_systems(Update, composer(ui))
        .run();
}
