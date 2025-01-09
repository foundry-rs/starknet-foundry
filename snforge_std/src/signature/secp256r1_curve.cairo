use starknet::secp256r1::Secp256r1Point;
use starknet::secp256_trait::{is_valid_signature, Secp256Trait, Secp256PointTrait};
use starknet::SyscallResultTrait;
use super::super::_cheatcode::typed_checked_cheatcode;
use super::SignError;
use snforge_std::signature::{KeyPair, KeyPairTrait, SignerTrait, VerifierTrait};

pub impl Secp256r1CurveKeyPairImpl of KeyPairTrait<u256, Secp256r1Point> {
    fn generate() -> KeyPair<u256, Secp256r1Point> {
        let (secret_key, pk_x, pk_y) = typed_checked_cheatcode::<
            'generate_ecdsa_keys', (u256, u256, u256)
        >(array!['Secp256r1'].span());

        let public_key = Secp256Trait::secp256_ec_new_syscall(pk_x, pk_y).unwrap_syscall().unwrap();

        KeyPair { secret_key, public_key }
    }

    fn from_secret_key(secret_key: u256) -> KeyPair<u256, Secp256r1Point> {
        if (secret_key == 0_u256
            || secret_key >= Secp256Trait::<Secp256r1Point>::get_curve_size()) {
            core::panic_with_felt252('invalid secret_key');
        }

        let generator = Secp256Trait::get_generator_point();

        let public_key = Secp256PointTrait::mul(generator, secret_key).unwrap_syscall();

        KeyPair { secret_key, public_key }
    }
}

pub impl Secp256r1CurveSignerImpl of SignerTrait<
    KeyPair<u256, Secp256r1Point>, u256, (u256, u256)
> {
    fn sign(
        self: KeyPair<u256, Secp256r1Point>, message_hash: u256
    ) -> Result<(u256, u256), SignError> {
        let mut input = array!['Secp256r1'];
        self.secret_key.serialize(ref input);
        message_hash.serialize(ref input);

        typed_checked_cheatcode::<
            'ecdsa_sign_message', Result<(u256, u256), SignError>
        >(input.span())
    }
}

pub impl Secp256r1CurveVerifierImpl of VerifierTrait<
    KeyPair<u256, Secp256r1Point>, u256, (u256, u256)
> {
    fn verify(
        self: KeyPair<u256, Secp256r1Point>, message_hash: u256, signature: (u256, u256)
    ) -> bool {
        let (r, s) = signature;
        is_valid_signature::<Secp256r1Point>(message_hash, r, s, self.public_key)
    }
}
