use bevy::ecs::system::Query;
use bevy_compose::{
    compose::{flex, lazy},
    Compose, StateComponent, UseState,
};

fn app() -> impl Compose {
    lazy(|mut count: UseState<i32>| {
        let (count, count_entity) = count.use_state(|| 0);

        flex((
            format!("High five count: {}", *count),
            flex("Up high!").on_click(move |mut count_query: Query<&mut StateComponent<i32>>| {
                if let Ok(mut count) = count_query.get_mut(count_entity) {
                    **count += 1
                }
            }),
            flex("Down low!").on_click(move |mut count_query: Query<&mut StateComponent<i32>>| {
                if let Ok(mut count) = count_query.get_mut(count_entity) {
                    **count -= 1
                }
            }),
        ))
    })
}

fn main() {
    bevy_compose::run(app);
}
