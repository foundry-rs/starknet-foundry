trait Signer<T> {
    fn sign(ref self: T, message_hash: u256) -> (u256, u256);
}

trait Verifier<T> {
    fn verify(ref self: T, message_hash: u256, signature: (u256, u256)) -> bool;
}
