#[derive(Copy, Drop)]
struct KeyPair<SK, PK> {
    secret_key: SK,
    public_key: PK,
}

trait KeyPairTrait<SK, PK> {
    fn generate() -> KeyPair<SK, PK>;
    fn from_secret_key(secret_key: SK) -> KeyPair<SK, PK>;
}

trait Signer<S, H, U> {
    fn sign(self: S, message_hash: H) -> (U, U);
}

// TODO: ref self?
trait Verifier<S, H, U> {
    fn verify(self: S, message_hash: H, signature: (U, U)) -> bool;
}
