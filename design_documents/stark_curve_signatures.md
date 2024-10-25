# `StarkCurve` signatures

## Context

Some users would like to have a way of generating and signing with the `stark curve`. It would be useful for testing
custom account implementations. 

## Existing solutions

Currently, it is possible only with the snForge I/O utilities which does not seem flexible. Also, to import such a signature
users have to generate it outside the Cairo and save it somewhere. Cheatcodes should simplify that process.

## Proposed solution

My proposal would be to introduce a `StarkCurveKeyPair` struct which would implement `sign` and `verify` methods.

```cairo
struct StarkCurveKeyPair {
    private_key: felt252,
    public_key: felt252 
}

trait StarkCurveKeyPairTrait {
    fn generate() -> StarkCurveKeyPair;
    fn sign(ref self: StarkCurveKeyPair, message_hash: felt252) -> (felt252, felt252);
    fn verify(ref self: StarkCurveKeyPair, message_hash: felt252, signature: (felt252, felt252)) -> bool;
}
```

## Example usage

```cairo
use snforge_std::{StarkCurveKeyPair, StarkCurveKeyPairTrait};

#[test]
fn test_stark_curve() {
    let mut key_pair = StarkCurveKeyPairTrait::generate();
    let message_hash = 12345;
    
    let signature = key_pair.sign(message_hash);
    
    assert(key_pair.verify(message_hash, signature), 'Signature is incorrect');
}
```
