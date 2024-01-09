# `Secp256r1 Curve`

Module containing the implementation of `KeyPairTrait` for the Secp256r1 curve.

> ⚠️ **Security Warning**
>
> Please note that cryptography in Starknet Foundry is still experimental and **has not been audited**.
>
> Use at your own risk!


## `Secp256r1CurveKeyPairImpl`

Implementation of `KeyPair` for the Secp256r1 curve.

```rust
struct KeyPair {
    secret_key: u256,
    public_key: Secp256r1Point,
}
```

## `Secp256r1CurveSignerImpl`

Implementation of the `SignerTrait` for the Secp256r1 curve.

> fn sign(self: KeyPair<u256, Secp256r1Point>, message_hash: u256) -> (u256, u256)


## `Secp256r1CurveVerifierImpl`

Implementation of the `VerifierTrait` for the Secp256r1 curve.

> fn verify(self: KeyPair<u256, Secp256r1Point>, message_hash: u256, signature: (u256, u256)) -> bool


## Example

```rust
use snforge_std::signature::KeyPairTrait;
use snforge_std::signature::secp256r1_curve::{Secp256r1CurveKeyPairImpl, Secp256r1CurveSignerImpl, Secp256r1CurveVerifierImpl};
use starknet::secp256r1::{Secp256r1Point, Secp256r1PointImpl};
use core::starknet::SyscallResultTrait;

#[test]
fn test_secp256r1_curve() {
    let key_pair = KeyPairTrait::<u256, Secp256r1Point>::generate();
    
    let msg_hash = 123456;
    let (r, s): (u256, u256) = key_pair.sign(msg_hash);

    let is_valid = key_pair.verify(msg_hash, (r, s));
    assert(is_valid, 'Signature should be valid');

    let key_pair2 = KeyPairTrait::<u256, Secp256r1Point>::from_secret_key(key_pair.secret_key);
    assert(key_pair.secret_key == key_pair2.secret_key, 'Secret keys should be equal');
    assert(key_pair.public_key.get_coordinates().unwrap_syscall() == key_pair2.public_key.get_coordinates().unwrap_syscall(), 'Public keys should be equal');
}
```
