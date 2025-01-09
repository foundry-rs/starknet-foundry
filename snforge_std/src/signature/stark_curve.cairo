use core::ecdsa::check_ecdsa_signature;
use core::ec::{EcPointImpl, EcPoint, stark_curve};
use super::super::_cheatcode::typed_checked_cheatcode;
use super::SignError;

use snforge_std::signature::{KeyPair, KeyPairTrait, SignerTrait, VerifierTrait};

pub impl StarkCurveKeyPairImpl of KeyPairTrait<felt252, felt252> {
    fn generate() -> KeyPair<felt252, felt252> {
        let (secret_key, public_key) = typed_checked_cheatcode::<
            'generate_stark_keys', (felt252, felt252)
        >(array![].span());

        KeyPair { secret_key, public_key }
    }

    fn from_secret_key(secret_key: felt252) -> KeyPair<felt252, felt252> {
        if (secret_key == 0) {
            core::panic_with_felt252('invalid secret_key');
        }

        let generator = EcPointImpl::new(stark_curve::GEN_X, stark_curve::GEN_Y).unwrap();

        let public_key: EcPoint = EcPointImpl::mul(generator, secret_key);

        let (pk_x, _pk_y) = public_key.try_into().unwrap().coordinates();

        KeyPair { secret_key, public_key: pk_x }
    }
}

pub impl StarkCurveSignerImpl of SignerTrait<
    KeyPair<felt252, felt252>, felt252, (felt252, felt252)
> {
    fn sign(
        self: KeyPair<felt252, felt252>, message_hash: felt252
    ) -> Result<(felt252, felt252), SignError> {
        typed_checked_cheatcode::<
            'stark_sign_message', Result<(felt252, felt252), SignError>
        >(array![self.secret_key, message_hash].span())
    }
}

pub impl StarkCurveVerifierImpl of VerifierTrait<
    KeyPair<felt252, felt252>, felt252, (felt252, felt252)
> {
    fn verify(
        self: KeyPair<felt252, felt252>, message_hash: felt252, signature: (felt252, felt252)
    ) -> bool {
        let (r, s) = signature;
        check_ecdsa_signature(message_hash, self.public_key, r, s)
    }
}
