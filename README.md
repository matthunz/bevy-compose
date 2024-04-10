# Bevy-compose

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

## Inspiration
This crate is inspired by [Xilem](https://github.com/linebender/xilem) and SwiftUI with its typed approach to reactivity.
