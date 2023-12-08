use array::{ArrayTrait, SpanTrait};
use traits::{Into, TryInto};

use starknet::secp256_trait::{Secp256Trait, Secp256PointTrait, is_valid_signature};
use starknet::secp256k1::{Secp256k1Point, Secp256k1Impl};
use starknet::secp256r1::{Secp256r1Point, Secp256r1Impl};
use starknet::{SyscallResult, SyscallResultTrait};
use starknet::testing::cheatcode;

use snforge_std::signature::key_pair::{KeyPair, KeyPairTrait, SignerTrait, VerifierTrait};

impl SecpCurveKeyPairImpl<Secp256Point, +Drop<Secp256Point>, impl Secp256Impl: Secp256Trait<Secp256Point>, impl Secp256PointImpl: Secp256PointTrait<Secp256Point>> of KeyPairTrait<u256, Secp256Point> {
    fn generate() -> KeyPair<u256, Secp256Point> {
        let curve = match_supported_curve::<Secp256Point>();

        let output = cheatcode::<'generate_ecdsa_keys'>(array![*curve[0]].span());

        let secret_key = to_u256(*output[0], *output[1]);
        let pk_x = to_u256(*output[2], *output[3]);
        let pk_y = to_u256(*output[4], *output[5]);

        let public_key = Secp256Impl::secp256_ec_new_syscall(pk_x, pk_y).unwrap_syscall().unwrap();

        KeyPair { secret_key, public_key }
    }

    fn from_secret_key(secret_key: u256) -> KeyPair<u256, Secp256Point> {
        if (secret_key == 0_u256 || secret_key >= Secp256Impl::get_curve_size()) {
            panic_with_felt252('invalid secret key');
        }

        let generator = Secp256Impl::get_generator_point();

        let public_key = Secp256PointImpl::mul(generator, secret_key).unwrap_syscall();

        KeyPair { secret_key, public_key }
    }
}

impl SecpCurveSignerImpl<Secp256Point, +Drop<Secp256Point>, impl Secp256Impl: Secp256Trait<Secp256Point>, impl Secp256PointImpl: Secp256PointTrait<Secp256Point>, H, +Drop<H>, impl HIntoU256: Into<H, u256>> of SignerTrait<KeyPair<u256, Secp256Point>, H, u256> {
    fn sign(self: KeyPair<u256, Secp256Point>, message_hash: H) -> (u256, u256) {
        let curve = match_supported_curve::<Secp256Point>();

        let (sk_low, sk_high) = from_u256(self.secret_key);
        let (msg_hash_low, msg_hash_high) = from_u256(message_hash.into());

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

impl SecpCurveVerifierImpl<Secp256Point, +Drop<Secp256Point>, impl Secp256Impl: Secp256Trait<Secp256Point>, impl Secp256PointImpl: Secp256PointTrait<Secp256Point>, H, impl HIntoU256: Into<H, u256>> of VerifierTrait<KeyPair<u256, Secp256Point>, H, u256> {
    fn verify(self: KeyPair<u256, Secp256Point>, message_hash: H, signature: (u256, u256)) -> bool {
        let (r, s) = signature;
        is_valid_signature::<Secp256Point>(message_hash.into(), r, s, self.public_key)
    }
}

fn match_supported_curve<Secp256Point, impl Secp256Impl: Secp256Trait<Secp256Point>, impl Secp256PointImpl: Secp256PointTrait<Secp256Point>>() -> Array<felt252> {
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
