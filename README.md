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


Reactive UI framework for Bevy

```rust
fn ui(count: Res<Count>) -> impl Compose {
    (
        format!("High five count: {}", count.0),
        flex("Up high!").on_click(|mut count: ResMut<Count>| count.0 += 1),
        flex("Down low!").on_click(|mut count: ResMut<Count>| count.0 -= 1),
    )
}
```

Components are also supported with `lazy(|player_query: Query<&mut Player>| { ... }`

## Inspiration
This crate is inspired by [Xilem](https://github.com/linebender/xilem), [Concoct](https://github.com/concoct-rs/concoct) and SwiftUI with its typed approach to reactivity.
