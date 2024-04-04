mod secp256k1_curve;
mod secp256r1_curve;
mod stark_curve;

#[derive(Copy, Drop)]
struct KeyPair<SK, PK> {
    /// A key that is used for signing the messages, it's 2 times shorter than the signature itself
    secret_key: SK,
    /// A (x, y) point on the elliptic curve used for verification of the signature
    public_key: PK,
}

trait KeyPairTrait<SK, PK> {
    /// Generates the private and public keys using the built-in random generator
    fn generate() -> KeyPair<SK, PK>;
    /// Derives the KeyPair (`secret_key` + `public_key`) using `secret_key`
    fn from_secret_key(secret_key: SK) -> KeyPair<SK, PK>;
}

trait SignerTrait<T, H, U> {
    /// Signs given message hash
    /// `self` - KeyPair used for signing
    /// `message_hash` - input to sign bounded by the curve type (u256 for 256bit curves, felt252 for StarkCurve)
    /// Returns the signature components (usually r,s tuple)
    fn sign(self: T, message_hash: H) -> U;
}

trait VerifierTrait<T, H, U> {
    /// `self` - KeyPair used for verifying
    /// `message_hash` - input to verify bounded by the curve type (u256 for 256bit curves, felt252 for StarkCurve)
    /// `signature` - the signature components (usually r,s tuple)
    /// Returns a boolean representing the validity of the signature
    fn verify(self: T, message_hash: H, signature: U) -> bool;
}

fn to_u256(low: felt252, high: felt252) -> u256 {
    u256 { low: low.try_into().unwrap(), high: high.try_into().unwrap() }
}

fn from_u256(x: u256) -> (felt252, felt252) {
    (x.low.into(), x.high.into())
}
