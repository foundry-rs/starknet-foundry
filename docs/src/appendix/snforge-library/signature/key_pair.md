# Key Pair

```rust
struct KeyPair<SK, PK> {
    secret_key: SK,
    public_key: PK,
}
```


## `KeyPairTrait`

```rust
trait KeyPairTrait<SK, PK> {
    fn generate() -> KeyPair<SK, PK>;
    fn from_secret_key(secret_key: SK) -> KeyPair<SK, PK>;
}
```


## `SignerTrait`

```rust
trait SignerTrait<T, H, U> {
    fn sign(self: T, message_hash: H) -> (U, U);
}
```


## `VerifierTrait`

```rust
trait VerifierTrait<T, H, U> {
    fn verify(self: T, message_hash: H, signature: (U, U)) -> bool;
}
```
