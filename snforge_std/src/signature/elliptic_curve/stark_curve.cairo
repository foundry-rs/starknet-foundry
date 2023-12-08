use starknet::testing::cheatcode;

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
use ecdsa::check_ecdsa_signature;
use snforge_std::signature::elliptic_curve::interface::{KeyPair, KeyPairTrait, Signer, Verifier};
use ec::{ EcPointTrait, NonZeroEcPoint };

impl StarkCurveKeyPairImpl<
    SK,
    +Drop<SK>,
    impl Felt252IntoSK: Into<felt252, SK>,
    impl SKIntoFelt252: Into<SK, felt252>,
    impl EcPointImpl: EcPointTrait,
> of KeyPairTrait<SK, NonZeroEcPoint> {
    fn generate() -> KeyPair<SK, NonZeroEcPoint> {
        let output = cheatcode::<'generate_stark_keys'>(array![].span());

        let secret_key: felt252 = *output[0];
        let public_key: felt252 = *output[1];

        let generator: EcPoint = EcPointImpl::new(ec::stark_curve::GEN_X, ec::stark_curve::GEN_Y).unwrap();
        let public_key: NonZeroEcPoint = EcPointImpl::new_from_x(public_key).unwrap().try_into().unwrap();

        let secret_key: SK = secret_key.into();

        KeyPair { secret_key, public_key }
    }

    fn from_secret_key(secret_key: SK) -> KeyPair<SK, NonZeroEcPoint> {
        let secret_key: felt252 = secret_key.into();
        let generator: EcPoint = EcPointImpl::new(ec::stark_curve::GEN_X, ec::stark_curve::GEN_Y).unwrap();

        let public_key: NonZeroEcPoint = EcPointImpl::mul(generator, secret_key).try_into().unwrap();
        let secret_key: SK = secret_key.into();
        
        KeyPair { secret_key, public_key }
    }
}


impl StarkCurveSignerImpl<
    SK,
    +Drop<SK>,
    impl SKIntoFelt252: Into<SK, felt252>,
    H,
    +Drop<H>,
    impl HIntoFelt252: Into<H, felt252>
> of Signer<KeyPair<SK, NonZeroEcPoint>, H, felt252> {
    fn sign(self: KeyPair<SK, NonZeroEcPoint>, message_hash: H) -> (felt252, felt252) {
        let output = cheatcode::<
            'stark_sign_message'
        >(array![self.secret_key.into(), message_hash.into()].span());

        if *output[0] == 1 {
            panic_with_felt252(*output[1]);
        }

        let r: felt252 = *output[1];
        let s: felt252 = *output[2];

        (r, s)
    }
}


impl StarkCurveVerifierImpl<
    SK,
    +Drop<SK>,
    impl EcPointImpl: EcPointTrait,
    H,
    +Drop<H>,
    impl HIntoFelt252: Into<H, felt252>,
    U,
    +Drop<U>,
    impl UIntoFelt252: Into<U, felt252>,
> of Verifier<KeyPair<SK, NonZeroEcPoint>, H, U> {
    fn verify(self: KeyPair<SK, NonZeroEcPoint>, message_hash: H, signature: (U, U)) -> bool {
        let (r, s) = signature;
        let (pk_x, pk_y): (felt252, felt252) = EcPointImpl::coordinates(self.public_key);
        check_ecdsa_signature(message_hash.into(), pk_x, r.into(), s.into())
    }
}
