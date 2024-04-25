use bevy_compose::{
    compose::{flex, lazy},
    Compose, UseState,
};

fn app() -> impl Compose {
    lazy(|mut count: UseState<i32>| {
        let (mut count, _count_entity) = count.use_state(|| 0);

        *count += 1;

        flex((format!("High five count: {}", *count), "Up high!"))
    })
}

fn main() {
    bevy_compose::run(app);
}
