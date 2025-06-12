use core::serde::Serde;
use core::option::OptionTrait;
use starknet::secp256k1::Secp256k1Point;
use starknet::secp256_trait::{is_valid_signature, Secp256Trait, Secp256PointTrait};
use starknet::{SyscallResultTrait};
use crate::cheatcode::execute_cheatcode_and_deserialize;
use super::SignError;

use snforge_std_compatibility::signature::{KeyPair, KeyPairTrait, SignerTrait, VerifierTrait};

pub impl Secp256k1CurveKeyPairImpl of KeyPairTrait<u256, Secp256k1Point> {
    fn generate() -> KeyPair<u256, Secp256k1Point> {
        let (secret_key, pk_x, pk_y) = execute_cheatcode_and_deserialize::<
            'generate_ecdsa_keys', (u256, u256, u256),
        >(array!['Secp256k1'].span());

        let public_key = Secp256Trait::secp256_ec_new_syscall(pk_x, pk_y).unwrap_syscall().unwrap();

        KeyPair { secret_key, public_key }
    }

    fn from_secret_key(secret_key: u256) -> KeyPair<u256, Secp256k1Point> {
        if (secret_key == 0_u256
            || secret_key >= Secp256Trait::<Secp256k1Point>::get_curve_size()) {
            core::panic_with_felt252('invalid secret_key');
        }

        let generator = Secp256Trait::get_generator_point();

        let public_key = Secp256PointTrait::mul(generator, secret_key).unwrap_syscall();

        KeyPair { secret_key, public_key }
    }
}

pub impl Secp256k1CurveSignerImpl of SignerTrait<
    KeyPair<u256, Secp256k1Point>, u256, (u256, u256),
> {
    fn sign(
        self: KeyPair<u256, Secp256k1Point>, message_hash: u256,
    ) -> Result<(u256, u256), SignError> {
        let mut input = array!['Secp256k1'];
        self.secret_key.serialize(ref input);
        message_hash.serialize(ref input);

        execute_cheatcode_and_deserialize::<'ecdsa_sign_message'>(input.span())
    }
}

pub impl Secp256k1CurveVerifierImpl of VerifierTrait<
    KeyPair<u256, Secp256k1Point>, u256, (u256, u256),
> {
    fn verify(
        self: KeyPair<u256, Secp256k1Point>, message_hash: u256, signature: (u256, u256),
    ) -> bool {
        let (r, s) = signature;
        is_valid_signature::<Secp256k1Point>(message_hash, r, s, self.public_key)
    }
}
