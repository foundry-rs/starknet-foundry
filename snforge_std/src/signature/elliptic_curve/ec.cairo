#[derive(Drop, Copy, Serde, PartialEq)]
enum EllipticCurve {
    Secp256k1,
    Secp256r1,
}
