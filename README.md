# BoltFFI

A high-performance multi-language bindings generator for Rust, up to 1,000x faster than UniFFI

<p align="center">
  <img src="docs/assets/demo.gif" width="700" />
</p>

Quick links: [User Guide](https://boltffi.dev/docs/overview) | [Tutorial](https://boltffi.dev/docs/tutorial) | [Getting Started](https://boltffi.dev/docs/getting-started)

## Performance

| Benchmark | BoltFFI | UniFFI | Speedup |
|-----------|--------:|-------:|--------:|
| noop | <1 ns | 1,416 ns | >1000x |
| echo_i32 | <1 ns | 1,416 ns | >1000x |
| counter_increment (1k calls) | 1,083 ns | 1,388,895 ns | 1,282x |
| generate_locations (1k structs) | 4,167 ns | 1,276,333 ns | 306x |
| generate_locations (10k structs) | 62,542 ns | 12,817,000 ns | 205x |

Full benchmark code: [benchmarks](./benchmarks)


## Why BoltFFI?

Serialization-based FFI is slow. Tools like UniFFI serialize every value to a byte buffer on each call. That overhead shows up when you're making thousands of FFI calls per second.

BoltFFI uses zero-copy where possible. Primitives pass as raw values. Structs with primitive fields pass as pointers to memory both sides can read directly. Only strings and collections go through encoding.

## What it does

Mark your Rust types with `#[data]` and functions with `#[export]`:

```rust
use boltffi::*;

#[data]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[export]
pub fn distance(a: Point, b: Point) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    (dx * dx + dy * dy).sqrt()
}
```

Run `boltffi pack`:

```bash
boltffi pack apple
# Produces: ./dist/YourCrate.xcframework + Package.swift

boltffi pack android
# Produces: ./dist/android/libs/*.so + Kotlin bindings
```

Use it from Swift or Kotlin:

```swift
let d = distance(a: Point(x: 0, y: 0), b: Point(x: 3, y: 4)) // 5.0
```

```kotlin
val d = distance(a = Point(x = 0.0, y = 0.0), b = Point(x = 3.0, y = 4.0)) // 5.0
```

The generated bindings use each language's idioms. Swift gets async/await. Kotlin gets coroutines. Errors become native exceptions.

## Supported languages

| Language | Status       |
|----------|--------------|
| Swift    | Full support |
| Kotlin   | Full support |
| WASM     | In progress  |
| Python   | Soon         |
| C#       | Soon         |

Want another language? [Open an issue](https://github.com/boltffi/boltffi/issues).

## Installation

```bash
cargo install boltffi_cli
```

Add to your `Cargo.toml`:

```toml
[dependencies]
boltffi = "0.1"

[lib]
crate-type = ["staticlib", "cdylib"]
```

## Documentation

- [Overview](https://boltffi.dev/docs/overview)
- [Getting Started](https://boltffi.dev/docs/getting-started)
- [Tutorial](https://boltffi.dev/docs/tutorial)
- [Types](https://boltffi.dev/docs/types)
- [Async](https://boltffi.dev/docs/async)
- [Streaming](https://boltffi.dev/docs/streaming)

## Alternative tools

Other tools that solve similar problems:

- [UniFFI](https://github.com/mozilla/uniffi-rs) - Mozilla's binding generator, uses serialization-based approach
- [Diplomat](https://github.com/rust-diplomat/diplomat) - Focused on C/C++ interop
- [cxx](https://github.com/dtolnay/cxx) - Safe C++/Rust interop

## Contributing
Contributions are warmly welcomed 🙌

- [File an issue](https://github.com/boltffi/boltffi/issues)
- [Submit a PR](https://github.com/boltffi/boltffi/pulls)

## License
BOLTFFI is released under the MIT license. See [LICENSE](./LICENSE) for more information.
