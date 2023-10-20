# `StarkCurveKeyPairTrait`

```rust
trait StarkCurveKeyPairTrait {
    fn generate() -> StarkCurveKeyPair;
    fn sign(
        ref self: StarkCurveKeyPair, message_hash: felt252
    ) -> Result<(felt252, felt252), felt252>;
    fn verify(
        ref self: StarkCurveKeyPair, message_hash: felt252, signature: (felt252, felt252)
    ) -> bool;
}
```

Defines an interface for [`StarkCurveKeyPair`](stark_curve_key_pair.md).

## Examples

```rust
use snforge_std::{ StarkCurveKeyPair, StarkCurveKeyPairTrait };

#[test]
fn simple_signing_flow() {
    let mut key_pair = StarkCurveKeyPairTrait::generate();
    let message_hash = 123456;

    let signature = key_pair.sign(message_hash).unwrap();
    assert(key_pair.verify(message_hash, signature), 'Wrong signature');
}
```

When signing and the `message_hash` is too big 
(`> 0x800000000000000000000000000000000000000000000000000000000000000`)
the `Result::Err('message_hash out of range')` error will be returned.

```rust
use snforge_std::{ StarkCurveKeyPair, StarkCurveKeyPairTrait };

#[test]
fn message_hash_too_big() {
    let mut key_pair = StarkCurveKeyPairTrait::generate();
    let max_felt = 3618502788666131213697322783095070105623107215331596699973092056135872020480;

    match key_pair.sign(max_felt) {
        Result::Ok(_) => panic_with_felt252('shouldve panicked'),
        Result::Err(msg) => {
            assert(msg == 'message_hash out of range', msg);
        }
    }
}
```