<div align="center">
  <h1>bevy-compose</h1>

 <a href="https://crates.io/crates/bevy-compose">
    <img src="https://img.shields.io/crates/v/bevy-compose?style=flat-square"
    alt="Crates.io version" />
  </a>
  <a href="https://docs.rs/bevy-compose">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
   <a href="https://github.com/matthunz/bevy-compose/actions">
    <img src="https://github.com/matthunz/bevy-compose/actions/workflows/rust.yml/badge.svg"
      alt="CI status" />
  </a>
</div>

<br />


Reactive bundle template plugin for Bevy.

This crate provides a framework for parallel reactive systems using the ECS.

```rust
use bevy::prelude::*;
use bevy_compose::{Template, TemplatePlugin};

#[derive(Clone, Copy, PartialEq, Component, Deref)]
struct Health(i32);

#[derive(Component, Deref)]
struct Damage(i32);

#[derive(Component)]
struct Zombie;

fn main() {
    App::new()
        .add_plugins(TemplatePlugin::default().add(Template::new(
            // Spawning a Zombie will spawn the following components:
            Zombie,
            (
                // This only runs once.
                || Health(100),
                // This runs every time a `Health` component is updated,
                // and it's guraranteed to run before other systems using the `Damage` component.
                |entity: In<Entity>, health_query: Query<&Health>| {
                    let health = health_query.get(*entity).unwrap();
                    Damage(**health * 2)
                },
            ),
        )))
        .add_systems(Startup, setup)
        .add_systems(PostUpdate, debug)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Zombie);
}

fn debug(query: Query<&Damage>) {
    for dmg in &query {
        dbg!(**dmg);
        // 200.
    }
}
```

## Inspiration
This crate is inspired by [Xilem](https://github.com/linebender/xilem), [Concoct](https://github.com/concoct-rs/concoct) and SwiftUI with its typed approach to reactivity.
