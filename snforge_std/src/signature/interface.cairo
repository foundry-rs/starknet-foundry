trait Signer<T> {
    fn sign(ref self: T, message_hash: felt252) -> Result<(felt252, felt252), felt252>;
}

trait Verifier<T> {
    fn verify(ref self: T, message_hash: felt252, signature: (felt252, felt252)) -> bool;
}
