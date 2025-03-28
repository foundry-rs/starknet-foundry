pub mod secp256k1_curve;
pub mod secp256r1_curve;
pub mod stark_curve;

#[derive(Copy, Drop)]
pub struct KeyPair<SK, PK> {
    /// A key that is used for signing the messages
    pub secret_key: SK,
    /// A (x, y) point on the elliptic curve used for verification of the signature
    pub public_key: PK,
}

pub trait KeyPairTrait<SK, PK> {
    /// Generates the private and public keys using the built-in random generator
    fn generate() -> KeyPair<SK, PK>;
    /// Derives the KeyPair (`secret_key` + `public_key`) using `secret_key`
    fn from_secret_key(secret_key: SK) -> KeyPair<SK, PK>;
}

pub trait SignerTrait<T, H, U> {
    /// Signs given message hash
    /// `self` - KeyPair used for signing
    /// `message_hash` - input to sign bounded by the curve type (u256 for 256bit curves, felt252
    /// for StarkCurve)
    /// Returns the signature components (usually r,s tuple) or error
    fn sign(self: T, message_hash: H) -> Result<U, SignError>;
}

pub trait VerifierTrait<T, H, U> {
    /// `self` - KeyPair used for verifying
    /// `message_hash` - input to verify bounded by the curve type (u256 for 256bit curves, felt252
    /// for StarkCurve)
    /// `signature` - the signature components (usually r,s tuple)
    /// Returns a boolean representing the validity of the signature
    fn verify(self: T, message_hash: H, signature: U) -> bool;
}

#[derive(Copy, Drop, Serde, PartialEq)]
pub enum SignError {
    InvalidSecretKey,
    HashOutOfRange,
}
