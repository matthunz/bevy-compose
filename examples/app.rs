use bevy_compose::{lazy, Compose, UseState};

fn app() -> impl Compose {
    lazy(|mut count: UseState<i32>| {
        let mut count = count.use_state(|| 0);

        dbg!(*count);

        *count += 1;
    })
}

fn main() {
    bevy_compose::run(app);
}
