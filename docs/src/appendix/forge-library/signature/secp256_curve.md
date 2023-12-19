# `Secp256 Curve`

This module contains the implementation of `KeyPairTrait` for the `Secp256k1` and `Secp256r1` curves.

> ⚠️ **Security Warning**
>
> Please note that cryptography in Starknet Foundry is still experimental and **has not been audited yet**.
>
> Use at your own risk!


## `Secp256CurveKeyPairImpl`

The `Secp256CurveKeyPairImpl` is an implementation of `KeyPair` with secret key being `u256` and private key being a `Secp256Point`.

The `Secp256Point` implements `Secp256PointTrait` from corelib.

```rust
struct KeyPair {
    secret_key: u256,
    public_key: Secp256Point,
}
```

A new key pair for the `Secp256r1` curve can be obtained as follows:

```rust
use snforge_std::signature::{KeyPair, KeyPairTrait, SignerTrait, VerifierTrait};
use snforge_std::signature::secp256_curve::{Secp256CurveKeyPairImpl, Secp256CurveSignerImpl, Secp256CurveVerifierImpl};
use starknet::secp256r1::{Secp256r1Impl, Secp256r1Point, Secp256r1PointImpl};

let key_pair_secp256r1 = KeyPairTrait::<u256, Secp256r1Point>::generate();
```

Or, if the `secret_key` is known in advance:

```rust
use snforge_std::signature::{KeyPair, KeyPairTrait, SignerTrait, VerifierTrait};
use snforge_std::signature::secp256_curve::{Secp256CurveKeyPairImpl, Secp256CurveSignerImpl, Secp256CurveVerifierImpl};
use starknet::secp256r1::{Secp256r1Impl, Secp256r1Point, Secp256r1PointImpl};

let secret_key = ...;
let key_pair_secp256r1 = KeyPairTrait::<u256, Secp256r1Point>::from_secret_key(secret_key);
```

## `Secp256CurveSignerImpl`

Implementation of the `SignerTrait` for the Secp256 curve.

> fn sign(self: KeyPair<u256, Secp256Point>, message_hash: H) -> (u256, u256)


## `Secp256CurveVerifierImpl`

Implementation of the `VerifierTrait` for the Secp256 curve.

> fn verify(self: KeyPair<u256, Secp256Point>, message_hash: H, signature: (u256, u256)) -> bool


## Example

```rust
use snforge_std::signature::{KeyPair, KeyPairTrait, SignerTrait, VerifierTrait};
use snforge_std::signature::secp256_curve::{Secp256CurveKeyPairImpl, Secp256CurveSignerImpl, Secp256CurveVerifierImpl};
use starknet::secp256k1::{Secp256k1Impl, Secp256k1Point, Secp256k1PointImpl};
use core::starknet::SyscallResultTrait;

#[test]
fn test() {
    let key_pair = KeyPairTrait::<u256, Secp256k1Point>::generate();
    
    let msg_hash = 123456;
    let (r, s): (u256, u256) = key_pair.sign(msg_hash);

    let is_valid = key_pair.verify(msg_hash, (r, s));
    assert(is_valid, 'Signature should be valid');

    let key_pair2 = KeyPairTrait::<u256, Secp256k1Point>::from_secret_key(key_pair.secret_key);
    assert(key_pair.secret_key == key_pair2.secret_key, 'Secret keys should be equal');
    assert(key_pair.public_key.get_coordinates().unwrap_syscall() == key_pair2.public_key.get_coordinates().unwrap_syscall(), 'Public keys should be equal');
}
```
