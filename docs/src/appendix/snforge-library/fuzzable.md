# `fuzzable` Module

Module containing `Fuzzable` trait needed for fuzz testing and its implementations for basic types.

## `Fuzzable`

```rust
pub trait Fuzzable<T, +Debug<T>> {
    fn blank() -> T;
    fn generate() -> T;
}
```

This trait is used by `snforge` to generate random data for fuzz testing.
Any type that is used as a parameter in a test function with the [`#[fuzzer]`](../../testing/test-attributes.md#fuzzer) attribute must implement this trait.

- `blank()` function returns an empty or default value that is used only for configuration runs. For instance, it returns `0` for numeric types.
- `generate()` function is used to return a random value of the given type. To implement this function, it is necessary to either use the [generate_arg](../cheatcodes/generate_arg.md) cheatcode,
which can uniformly generate a random number within a specified range, or use a `Fuzzable` implementation from a different type.

## Example

Implementation for a custom type `Message`:

```rust
use core::num::traits::Bounded;
use snforge_std::fuzzable::{Fuzzable, generate_arg};

#[derive(Debug, Drop)]
struct Message {
    id: u64,
    text: ByteArray
}

impl FuzzableMessage of Fuzzable<Message> {
    fn blank() -> Message {
        Message {
            id: 0,
            text: ""
        }
    }

    fn generate() -> Message {
        Message {
            id: generate_arg(0, Bounded::<u64>::MAX),
            text: Fuzzable::<ByteArray>::generate()
        }
    }
}
```
