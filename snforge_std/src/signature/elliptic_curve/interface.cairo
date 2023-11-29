trait Signer<T> {
    fn sign(ref self: T, message_hash: u256) -> Result<(u256, u256), felt252>;
}

trait Verifier<T> {
    fn verify(ref self: T, message_hash: u256, signature: (u256, u256)) -> bool;
}
