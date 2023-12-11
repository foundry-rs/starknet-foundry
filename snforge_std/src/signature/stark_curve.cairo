use core::starknet::secp256_trait::Secp256PointTrait;
use array::{ArrayTrait, SpanTrait};
use traits::{Into, TryInto};

use starknet::{SyscallResult, SyscallResultTrait};
use starknet::testing::cheatcode;

use ec::{EcPointImpl, EcPointTrait, NonZeroEcPoint, EcPointNeg};
use ecdsa::check_ecdsa_signature;

use snforge_std::signature::key_pair::{KeyPair, KeyPairTrait, SignerTrait, VerifierTrait};

const PRIME_DIV_2: u256 = 0x400000000000008800000000000000000000000000000000000000000000000;

// TODO: either that or always return y < PRIME / 2
// Note that the points returned as public keys from
// generate() and from_secret_key(secret_key: felt252)
// might have different y coordinates
impl StarkCurveKeyPairImpl<
    // impl EcPointImpl: EcPointTrait
> of KeyPairTrait<felt252, NonZeroEcPoint> {
    fn generate() -> KeyPair<felt252, NonZeroEcPoint> {
        let output = cheatcode::<'generate_stark_keys'>(array![].span());

        let secret_key = *output[0];
        let pk_x = *output[1];

        // EcPointImpl::new_from_x deterministically returns y < PRIME / 2
        // https://github.com/starkware-libs/cairo/blob/e6c338c3a42fd7a50be8c27fbcc1c40173103094/crates/cairo-lang-sierra-to-casm/src/invocations/ec.rs#L214
        let public_key = EcPointImpl::new_from_x(pk_x).unwrap().try_into().unwrap();

        KeyPair { secret_key, public_key }
    }

    fn from_secret_key(secret_key: felt252) -> KeyPair<felt252, NonZeroEcPoint> {
        if (secret_key == 0) {
            panic_with_felt252('invalid secret_key');
        }

        let generator = EcPointImpl::new(ec::stark_curve::GEN_X, ec::stark_curve::GEN_Y).unwrap();

        let mut public_key: EcPoint = EcPointImpl::mul(generator, secret_key);

        // to deterministically return y < PRIME / 2, flip the coordinate
        let (pk_x, pk_y) = public_key.try_into().unwrap().coordinates();
        if pk_y.into() >= PRIME_DIV_2 {
            public_key = -public_key;
        }

        let public_key: NonZeroEcPoint = public_key.try_into().unwrap();

        KeyPair { secret_key, public_key }
    }
}

impl StarkCurveSignerImpl<
    H, +Drop<H>, impl HIntoFelt252: Into<H, felt252>
> of SignerTrait<KeyPair<felt252, NonZeroEcPoint>, H, felt252> {
    fn sign(self: KeyPair<felt252, NonZeroEcPoint>, message_hash: H) -> (felt252, felt252) {
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
    impl EcPointImpl: EcPointTrait, H, +Drop<H>, impl HIntoFelt252: Into<H, felt252>
> of VerifierTrait<KeyPair<felt252, NonZeroEcPoint>, H, felt252> {
    fn verify(
        self: KeyPair<felt252, NonZeroEcPoint>, message_hash: H, signature: (felt252, felt252)
    ) -> bool {
        let (r, s) = signature;
        let (pk_x, pk_y) = EcPointImpl::coordinates(self.public_key);

        check_ecdsa_signature(message_hash.into(), pk_x, r, s)
    }
}
