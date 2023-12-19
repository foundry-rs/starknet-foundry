# `signature` Module

Module containing struct and interface for creating `ecdsa` signatures

* [`Key Pair`](signature/key_pair.md) - keeps the `KeyPair` struct and connected traits
* [`STARK Curve`](signature/stark_curve.md) - implementation of `KeyPair` for the STARK curve
* [`Secp256 Curve`](signature/secp256_curve.md) - implementation of `KeyPair` for `Secp256k1` and `Secp256r1` curves


> ⚠️ **Security Warning**
>
> Please note that cryptography in Starknet Foundry is still experimental and **has not been audited yet**.
>
> Use at your own risk!