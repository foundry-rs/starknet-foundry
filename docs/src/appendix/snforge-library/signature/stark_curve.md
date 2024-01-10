# `STARK Curve`

Module containing the implementation of `KeyPairTrait` for the [STARK Curve](https://docs.starknet.io/documentation/architecture_and_concepts/Cryptography/stark-curve/).

> ⚠️ **Security Warning**
>
> Please note that cryptography in Starknet Foundry is still experimental and **has not been audited**.
>
> Use at your own risk!


## `StarkCurveKeyPairImpl`

Implementation of `KeyPair` for the STARK curve.

```rust
struct KeyPair {
    secret_key: felt252,
    public_key: felt252,
}
```


## `StarkCurveSignerImpl`

Implementation of the `SignerTrait` for the STARK curve.

> fn sign(self: KeyPair<felt252, felt252>, message_hash: felt252) -> (felt252, felt252)


## `StarkCurveVerifierImpl`

Implementation of the `VerifierTrait` for the STARK curve.

> fn verify(self: KeyPair<felt252, felt252>, message_hash: felt252, signature: (felt252, felt252)) -> bool


## Example

```rust
use snforge_std::signature::KeyPairTrait;
use snforge_std::signature::stark_curve::{StarkCurveKeyPairImpl, StarkCurveSignerImpl, StarkCurveVerifierImpl};

#[test]
fn test_stark_curve() {
    let key_pair = KeyPairTrait::<felt252, felt252>::generate();
    
    let msg_hash = 123456;
    let (r, s): (felt252, felt252) = key_pair.sign(msg_hash);

    let is_valid = key_pair.verify(msg_hash, (r, s));
    assert(is_valid, 'Signature should be valid');

    let key_pair2 = KeyPairTrait::<felt252, felt252>::from_secret_key(key_pair.secret_key);

    assert(key_pair.secret_key == key_pair2.secret_key, 'Secret keys should be equal');
    assert(key_pair.public_key == key_pair2.public_key, 'Public keys should be equal');
}
```
