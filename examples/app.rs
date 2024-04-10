use bevy::prelude::*;
use bevy_compose::{compose::flex, Compose, ComposePlugin};

#[derive(Resource)]
struct Count(i32);

fn ui(count: Res<Count>) -> impl Compose {
    (
        format!("High five count: {}", count.0),
        flex("Up high!")
            .on_click(|mut count: ResMut<Count>| count.0 += 1)
            .on_hover(|| {
                dbg!("Hover!");
            }),
        flex("Down low!").on_click(|mut count: ResMut<Count>| count.0 -= 1),
    )
}

fn main() {
    let mut app = App::new();

    app.world.insert_resource(Count(0));
    app.world
        .spawn((Camera2dBundle::default(), IsDefaultUiCamera));

    app.add_plugins((DefaultPlugins, ComposePlugin::new(ui)))
        .run();
}
