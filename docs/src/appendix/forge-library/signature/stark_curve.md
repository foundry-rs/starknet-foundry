# `STARK Curve`

This module contains the implementation of `KeyPairTrait` for the [STARK Curve](https://docs.starknet.io/documentation/architecture_and_concepts/Cryptography/stark-curve/).

> ⚠️ **Security Warning**
>
> Please note that cryptography in Starknet Foundry is still experimental and **has not been audited yet**.
>
> Use at your own risk!


## `StarkCurveKeyPairImpl`

The `StarkCurveKeyPairImpl` is an implementation of `KeyPair` with both the secret and private keys being of type `felt252`.

```rust
struct KeyPair {
    secret_key: felt252,
    public_key: felt252,
}
```

A new key pair for the STARK curve can be obtained as follows:

```rust
use snforge_std::signature::key_pair::{KeyPair, KeyPairTrait};
use snforge_std::signature::stark_curve::StarkCurveKeyPairImpl;

let key_pair = KeyPairTrait::<felt252, felt252>::generate();
```

Or, if the `secret_key` is known in advance:

```rust
use snforge_std::signature::key_pair::{KeyPair, KeyPairTrait};
use snforge_std::signature::stark_curve::StarkCurveKeyPairImpl;

let secret_key = ...;
let key_pair = KeyPairTrait::<felt252, felt252>::from_secret_key(secret_key);
```


## `StarkCurveSignerImpl`

Implementation of the `SignerTrait` for the STARK curve.

> fn sign(self: KeyPair<felt252, felt252>, message_hash: H) -> (felt252, felt252)


## `StarkCurveVerifierImpl`

Implementation of the `VerifierTrait` for the STARK curve.

> fn verify(self: KeyPair<felt252, felt252>, message_hash: H, signature: (felt252, felt252)) -> bool


## Example

```rust
use snforge_std::signature::key_pair::{KeyPair, KeyPairTrait, SignerTrait, VerifierTrait};
use snforge_std::signature::stark_curve::{StarkCurveKeyPairImpl, StarkCurveSignerImpl, StarkCurveVerifierImpl};

#[test]
fn test_stark_curve() {
    let key_pair = KeyPairTrait::<felt252, felt252>::generate();
    
    let msg_hash = 123456;
    let (r, s) = key_pair.sign(msg_hash);

    let is_valid = key_pair.verify(msg_hash, (r, s));
    assert(is_valid, 'Signature should be valid');

    let key_pair2 = KeyPairTrait::<felt252, felt252>::from_secret_key(key_pair.secret_key);

    assert(key_pair.secret_key == key_pair2.secret_key, 'Secret keys should be equal');
    assert(key_pair.public_key == key_pair2.public_key, 'Public keys should be equal');
}
```
