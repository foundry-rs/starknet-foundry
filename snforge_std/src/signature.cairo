mod secp256k1_curve;
mod secp256r1_curve;
mod stark_curve;

#[derive(Copy, Drop)]
struct KeyPair<SK, PK> {
    secret_key: SK,
    public_key: PK,
}

trait KeyPairTrait<SK, PK> {
    fn generate() -> KeyPair<SK, PK>;
    fn from_secret_key(secret_key: SK) -> KeyPair<SK, PK>;
}

trait SignerTrait<T, H, U> {
    fn sign(self: T, message_hash: H) -> U;
}

trait VerifierTrait<T, H, U> {
    fn verify(self: T, message_hash: H, signature: U) -> bool;
}

fn to_u256(low: felt252, high: felt252) -> u256 {
    u256 { low: low.try_into().unwrap(), high: high.try_into().unwrap() }
}

fn from_u256(x: u256) -> (felt252, felt252) {
    (x.low.into(), x.high.into())
}
