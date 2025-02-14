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

- `blank()` returns an empty or default value. The specific value used does not matter much, as it is only used for configuration runs. For types that implement the `Default` trait, it is recommended to return `Default::default()`.
- `generate()` function is used to return a random value of the given type. To implement this function, it is necessary to either use a `Fuzzable` implementation from a different type,
or use the [generate_arg](../cheatcodes/generate_arg.md) cheatcode, which can uniformly generate a random number within a specified range.

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
            // Implementation may consist of:
            // Specifying a concrete value for the field
            id: 0,
            // Or using default value from `Default` trait
            text: Default::default()
        }
    }

    fn generate() -> Message {
        Message {
            // Using `generate_arg` cheatcode
            id: generate_arg(0, Bounded::<u64>::MAX),
            // Or calling `generate` function on a type that already implements `Fuzzable`
            text: Fuzzable::<ByteArray>::generate()
        }
    }
}
```
