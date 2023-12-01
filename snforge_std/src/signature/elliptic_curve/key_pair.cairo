use starknet::testing::cheatcode;
use super::interface::Signer;
use super::interface::Verifier;

use snforge_std::PrintTrait;
use array::ArrayTrait;
use array::SpanTrait;
use traits::Into;
use traits::TryInto;
use serde::Serde;
use starknet::secp256_trait::{Secp256Trait, Secp256PointTrait, is_valid_signature};
use starknet::secp256k1::{Secp256k1Point, Secp256k1Impl};
use starknet::secp256r1::{Secp256r1Point, Secp256r1Impl};
use starknet::{SyscallResult, SyscallResultTrait};

#[derive(Copy, Drop)]
struct KeyPair<T> {
    secret_key: u256,
    public_key: T,
}

#[generate_trait]
impl KeyPairImpl<
    Secp256Point,
    impl Secp256Impl: Secp256Trait<Secp256Point>,
    impl Secp256PointImpl: Secp256PointTrait<Secp256Point>
> of KeyPairTrait<Secp256Point> {
    fn generate() -> KeyPair<Secp256Point> {
        let curve = match_supported_curve::<Secp256Point>();

        let output: Span<felt252> = cheatcode::<'generate_ecdsa_keys'>(array![*curve[0]].span());

        let secret_key = to_u256(*output[0], *output[1]);
        let x = to_u256(*output[2], *output[3]);
        let y = to_u256(*output[4], *output[5]);

        let public_key = Secp256Impl::secp256_ec_new_syscall(x, y).unwrap_syscall().unwrap();

        KeyPair { secret_key, public_key }
    }

    fn from_secret_key(secret_key: u256) -> KeyPair<Secp256Point> {
        let curve = match_supported_curve::<Secp256Point>();
        let (sk_low, sk_high) = from_u256(secret_key);

        let output = cheatcode::<'get_ecdsa_public_key'>(array![*curve[0], sk_low, sk_high].span());

        if *output[0] == 1 {
            panic_with_felt252(*output[1]);
        }

        let x = to_u256(*output[1], *output[2]);
        let y = to_u256(*output[3], *output[4]);

        let public_key = Secp256Impl::secp256_ec_new_syscall(x, y).unwrap_syscall().unwrap();

        KeyPair { secret_key, public_key }
    }
}


impl KeyPairSigner<
    Secp256Point,
    +Drop<Secp256Point>,
    impl Secp256Impl: Secp256Trait<Secp256Point>,
    impl Secp256PointImpl: Secp256PointTrait<Secp256Point>
> of Signer<KeyPair<Secp256Point>> {
    fn sign(self: KeyPair<Secp256Point>, message_hash: u256) -> (u256, u256) {
        let curve = match_supported_curve::<Secp256Point>();
        let (sk_low, sk_high) = from_u256(self.secret_key);
        let (msg_hash_low, msg_hash_high) = from_u256(message_hash);

        let output = cheatcode::<
            'ecdsa_sign_message'
        >(array![*curve[0], sk_low, sk_high, msg_hash_low, msg_hash_high].span());

        if *output[0] == 1 {
            panic_with_felt252(*output[1]);
        }

        let r = to_u256(*output[1], *output[2]);
        let s = to_u256(*output[3], *output[4]);

        (r, s)
    }
}

impl KeyPairVerifier<
    Secp256Point,
    +Copy<Secp256Point>,
    +Drop<Secp256Point>,
    impl Secp256Impl: Secp256Trait<Secp256Point>,
    impl Secp256PointImpl: Secp256PointTrait<Secp256Point>
> of Verifier<KeyPair<Secp256Point>> {
    fn verify(self: KeyPair<Secp256Point>, message_hash: u256, signature: (u256, u256)) -> bool {
        let (r, s) = signature;
        is_valid_signature::<Secp256Point>(message_hash, r, s, self.public_key)
    }
}

fn match_supported_curve<
    Secp256Point,
    impl Secp256Impl: Secp256Trait<Secp256Point>,
    impl Secp256PointImpl: Secp256PointTrait<Secp256Point>
>() -> Array<
    felt252
> {
    let curve_size = Secp256Impl::get_curve_size();
    let generator = Secp256Impl::get_generator_point().get_coordinates().unwrap_syscall();

    let mut curve = array![];
    if curve_size == Secp256k1Impl::get_curve_size()
        && generator == Secp256k1Impl::get_generator_point().get_coordinates().unwrap_syscall() {
        curve = array!['Secp256k1'];
    } else if curve_size == Secp256r1Impl::get_curve_size()
        && generator == Secp256r1Impl::get_generator_point().get_coordinates().unwrap_syscall() {
        curve = array!['Secp256r1'];
    } else {
        panic(array!['Currently only Secp256k1 and', 'Secp256r1 curves are supported']);
    }
    curve
}

fn to_u256(low: felt252, high: felt252) -> u256 {
    u256 { low: low.try_into().unwrap(), high: high.try_into().unwrap() }
}

fn from_u256(x: u256) -> (felt252, felt252) {
    (x.low.into(), x.high.into())
}
