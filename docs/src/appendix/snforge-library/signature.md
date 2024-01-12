# `signature` Module

Module containing `KeyPair` struct and interface for creating `ecdsa` signatures.

* [`stark_curve`](signature/stark_curve.md) - implementation of `KeyPairTrait` for the STARK curve
* [`secp256k1_curve`](signature/secp256k1_curve.md) - implementation of `KeyPairTrait` for Secp256k1 Curve
* [`secp256r1_curve`](signature/secp256r1_curve.md) - implementation of `KeyPairTrait` for Secp256r1 Curve


> ⚠️ **Security Warning**
>
> Please note that cryptography in Starknet Foundry is still experimental and **has not been audited**.
>
> Use at your own risk!


## `KeyPair`

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
    fn sign(self: T, message_hash: H) -> U;
}
```


## `VerifierTrait`

```rust
trait VerifierTrait<T, H, U> {
    fn verify(self: T, message_hash: H, signature: U) -> bool;
}
```
