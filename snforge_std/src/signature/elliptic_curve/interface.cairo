trait Signer<T> {
    fn sign(self: T, message_hash: u256) -> (u256, u256);
}

trait Verifier<T> {
    fn verify(self: T, message_hash: u256, signature: (u256, u256)) -> bool;
}
