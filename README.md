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


Parallel reactivity for Bevy.

This crate provides a framework for UI and other reactive systems using the ECS.
Components can be created with `lazy` and run in parallel like regular systems (they can even use `Local` and other system parameters).

```rust
#[derive(Component, Deref, DerefMut)]
struct Count(i32);

fn app() -> impl Compose {
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
```

## Inspiration
This crate is inspired by [Xilem](https://github.com/linebender/xilem), [Concoct](https://github.com/concoct-rs/concoct) and SwiftUI with its typed approach to reactivity.
