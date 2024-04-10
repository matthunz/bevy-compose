use bevy::prelude::*;
use bevy_compose::{
    compose::{flex, memo},
    Compose, ComposePlugin,
};

#[derive(Resource)]
struct Count(i32);

fn ui(count: Res<Count>) -> impl Compose {
    memo(
        count.0,
        flex((
            format!("High five count: {}", count.0),
            flex("Up high!")
                .on_click(|mut count: ResMut<Count>| count.0 += 1)
                .on_hover(|| {
                    dbg!("Hover!");
                }),
            flex("Down low!").on_click(|mut count: ResMut<Count>| count.0 -= 1),
            if count.0 == 2 {
                Some("The number 2!")
            } else {
                None
            },
        )),
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
