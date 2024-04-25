use bevy::ecs::system::Local;
use bevy_compose::{lazy, Compose};

fn app() -> impl Compose {
    lazy(|mut x: Local<i32>| {
        dbg!(*x);
        *x += 1;

        lazy(|mut y: Local<i32>| {
            dbg!(*y);
            *y += 1;
        })
    })
}

fn main() {
    bevy_compose::run(app());
}
