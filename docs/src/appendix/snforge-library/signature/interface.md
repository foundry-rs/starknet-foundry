# Interface

## `Signer`

Trait representing the `Signer` interface.

```rust
trait Signer<T> {
    fn sign(ref self: T, message_hash: felt252) -> Result<(felt252, felt252), felt252>;
}
```

## `Verifier`

Trait representing the `Verifier` interface.

```rust
trait Verifier<T> {
    fn verify(ref self: T, message_hash: felt252, signature: (felt252, felt252)) -> bool;
}
```