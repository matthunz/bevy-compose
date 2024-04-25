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
fn app() -> impl Compose {
    lazy(|mut count: UseState<i32>| {
        let (mut count, _count_entity) = count.use_state(|| 0);

        dbg!(*count);

        *count += 1;
    })
}

fn main() {
    bevy_compose::run(app);
}
```

## Inspiration
This crate is inspired by [Xilem](https://github.com/linebender/xilem), [Concoct](https://github.com/concoct-rs/concoct) and SwiftUI with its typed approach to reactivity.
