use starknet::testing::cheatcode;
use ecdsa::check_ecdsa_signature;
use super::interface::Signer;
use super::interface::Verifier;

#[derive(Copy, Drop)]
struct StarkCurveKeyPair {
    private_key: felt252,
    public_key: felt252
}

#[generate_trait]
impl StarkCurveKeyPairImpl of StarkCurveKeyPairTrait {
    fn generate() -> StarkCurveKeyPair {
        let output = cheatcode::<'generate_ecdsa_keys'>(array![].span());

        StarkCurveKeyPair { private_key: *output[0], public_key: *output[1] }
    }

    fn from_private_key(private_key: felt252) -> StarkCurveKeyPair {
        let output = cheatcode::<'get_public_key'>(array![private_key].span());

        StarkCurveKeyPair { private_key, public_key: *output[0] }
    }
}

impl StarkCurveKeyPairSigner of Signer<StarkCurveKeyPair> {
    fn sign(
        ref self: StarkCurveKeyPair, message_hash: felt252
    ) -> Result<(felt252, felt252), felt252> {
        let output = cheatcode::<'ecdsa_sign_message'>(
            array![self.private_key, message_hash].span()
        );

        if *output[0] == 0 {
            Result::Ok((*output[1], *output[2]))
        } else if *output[0] == 1 {
            Result::Err(*output[1])
        } else {
            panic_with_felt252('Should not be reached')
        }
    }
}

impl StarkCurveKeyPairVerifier of Verifier<StarkCurveKeyPair> {
    fn verify(
        ref self: StarkCurveKeyPair, message_hash: felt252, signature: (felt252, felt252)
    ) -> bool {
        let (r, s) = signature;
        check_ecdsa_signature(message_hash, self.public_key, r, s)
    }
}
