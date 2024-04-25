use bevy::{
    ecs::{component::Component, system::Query},
    prelude::{Deref, DerefMut},
};
use bevy_compose::{
    compose::{flex, lazy},
    Compose, UseState,
};

#[derive(Component, Deref, DerefMut)]
struct Count(i32);

fn counter() -> impl Compose {
    lazy(|mut count: UseState<Count>| {
        let (count, count_entity) = count.use_state(|| Count(0));

        flex((
            format!("High five count: {}", **count),
            flex("Up high!").on_click(move |mut count_query: Query<&mut Count>| {
                if let Ok(mut count) = count_query.get_mut(count_entity) {
                    **count += 1
                }
            }),
            flex("Down low!").on_click(move |mut count_query: Query<&mut Count>| {
                if let Ok(mut count) = count_query.get_mut(count_entity) {
                    **count -= 1
                }
            }),
        ))
    })
}

fn app() -> impl Compose {
    flex((lazy(|| counter()), lazy(|| counter())))
}

fn main() {
    bevy_compose::run(app);
}
