use bevy::ecs::system::Local;
use bevy_compose::{effect, lazy, Compose};

fn app() -> impl Compose {
    lazy(|| {
        effect(0, |mut x: Local<i32>| {
            dbg!(*x);
            *x += 1;
        })
    })
}

fn main() {
    bevy_compose::run(app());
}
