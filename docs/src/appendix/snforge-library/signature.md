# `signature` Module

Module containing `KeyPair` struct and interface for creating `ecdsa` signatures.

* `signature::stark_curve` - implementation of `KeyPairTrait` for the STARK curve
* `signature::secp256k1_curve` - implementation of `KeyPairTrait` for Secp256k1 Curve
* `signature::secp256r1_curve` - implementation of `KeyPairTrait` for Secp256r1 Curve


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
trait SignerTrait<T, H, U, E> {
    fn sign(self: T, message_hash: H) -> Result<U, E> ;
}
```


## `VerifierTrait`

```rust
trait VerifierTrait<T, H, U> {
    fn verify(self: T, message_hash: H, signature: U) -> bool;
}
```

## Example

```rust
use snforge_std::signature::KeyPairTrait;

use snforge_std::signature::secp256r1_curve::{Secp256r1CurveKeyPairImpl, Secp256r1CurveSignerImpl, Secp256r1CurveVerifierImpl};
use snforge_std::signature::secp256k1_curve::{Secp256k1CurveKeyPairImpl, Secp256k1CurveSignerImpl, Secp256k1CurveVerifierImpl};
use snforge_std::signature::stark_curve::{StarkCurveKeyPairImpl, StarkCurveSignerImpl, StarkCurveVerifierImpl};

use starknet::secp256r1::{Secp256r1Point, Secp256r1PointImpl};
use starknet::secp256k1::{Secp256k1Point, Secp256k1PointImpl};
use core::starknet::SyscallResultTrait;

#[test]
fn test_using_curves() {
    // Secp256r1
    let key_pair = KeyPairTrait::<u256, Secp256r1Point>::generate();
    let (r, s): (u256, u256) = key_pair.sign(msg_hash).unwrap();
    let is_valid = key_pair.verify(msg_hash, (r, s));
    
    // Secp256k1
    let key_pair = KeyPairTrait::<u256, Secp256k1Point>::generate();
    let (r, s): (u256, u256) = key_pair.sign(msg_hash).unwrap();
    let is_valid = key_pair.verify(msg_hash, (r, s));
    
    // StarkCurve
    let key_pair = KeyPairTrait::<felt252, felt252>::generate();
    let (r, s): (felt252, felt252) = key_pair.sign(msg_hash).unwrap();
    let is_valid = key_pair.verify(msg_hash, (r, s));
}
```
