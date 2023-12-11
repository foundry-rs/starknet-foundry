use array::{ArrayTrait, SpanTrait};
use ecdsa::check_ecdsa_signature;
use traits::{Into, TryInto};
use ec::{EcPointImpl};

use starknet::testing::cheatcode;

use snforge_std::signature::key_pair::{KeyPair, KeyPairTrait, SignerTrait, VerifierTrait};

impl StarkCurveKeyPairImpl of KeyPairTrait<felt252, felt252> {
    fn generate() -> KeyPair<felt252, felt252> {
        let output = cheatcode::<'generate_stark_keys'>(array![].span());

        let secret_key = *output[0];
        let public_key = *output[1];

        KeyPair { secret_key, public_key }
    }

    fn from_secret_key(secret_key: felt252) -> KeyPair<felt252, felt252> {
        if (secret_key == 0) {
            panic_with_felt252('invalid secret_key');
        }

        let generator = EcPointImpl::new(ec::stark_curve::GEN_X, ec::stark_curve::GEN_Y).unwrap();

        let public_key: EcPoint = EcPointImpl::mul(generator, secret_key);

        let (pk_x, pk_y) = public_key.try_into().unwrap().coordinates();

        KeyPair { secret_key, public_key: pk_x }
    }
}

impl StarkCurveSignerImpl<
    H, +Drop<H>, impl HIntoFelt252: Into<H, felt252>
> of SignerTrait<KeyPair<felt252, felt252>, H, felt252> {
    fn sign(self: KeyPair<felt252, felt252>, message_hash: H) -> (felt252, felt252) {
        let output = cheatcode::<
            'stark_sign_message'
        >(array![self.secret_key, message_hash.into()].span());
        if *output[0] == 1 {
            panic_with_felt252(*output[1]);
        }

        let r = *output[1];
        let s = *output[2];

        (r, s)
    }
}

impl StarkCurveVerifierImpl<
    H, +Drop<H>, impl HIntoFelt252: Into<H, felt252>
> of VerifierTrait<KeyPair<felt252, felt252>, H, felt252> {
    fn verify(
        self: KeyPair<felt252, felt252>, message_hash: H, signature: (felt252, felt252)
    ) -> bool {
        let (r, s) = signature;
        check_ecdsa_signature(message_hash.into(), self.public_key, r, s)
    }
}
