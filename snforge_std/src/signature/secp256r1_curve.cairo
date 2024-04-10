use starknet::secp256_trait::{is_valid_signature};
use starknet::secp256r1::{Secp256r1Point, Secp256r1Impl, Secp256r1PointImpl};
use starknet::{SyscallResultTrait};
use starknet::testing::cheatcode;
use super::super::_cheatcode::handle_cheatcode;
use snforge_std::signature::{KeyPair, KeyPairTrait, SignerTrait, VerifierTrait, to_u256, from_u256};

impl Secp256r1CurveKeyPairImpl of KeyPairTrait<u256, Secp256r1Point> {
    fn generate() -> KeyPair<u256, Secp256r1Point> {
        let output = handle_cheatcode(
            cheatcode::<'generate_ecdsa_keys'>(array!['Secp256r1'].span())
        );

        let secret_key = to_u256(*output[0], *output[1]);
        let pk_x = to_u256(*output[2], *output[3]);
        let pk_y = to_u256(*output[4], *output[5]);

        let public_key = Secp256r1Impl::secp256_ec_new_syscall(pk_x, pk_y)
            .unwrap_syscall()
            .unwrap();

        KeyPair { secret_key, public_key }
    }

    fn from_secret_key(secret_key: u256) -> KeyPair<u256, Secp256r1Point> {
        if (secret_key == 0_u256 || secret_key >= Secp256r1Impl::get_curve_size()) {
            core::panic_with_felt252('invalid secret_key');
        }

        let generator = Secp256r1Impl::get_generator_point();

        let public_key = Secp256r1PointImpl::mul(generator, secret_key).unwrap_syscall();

        KeyPair { secret_key, public_key }
    }
}

impl Secp256r1CurveSignerImpl of SignerTrait<KeyPair<u256, Secp256r1Point>, u256, (u256, u256)> {
    fn sign(self: KeyPair<u256, Secp256r1Point>, message_hash: u256) -> (u256, u256) {
        let (sk_low, sk_high) = from_u256(self.secret_key);
        let (msg_hash_low, msg_hash_high) = from_u256(message_hash);

        let output = handle_cheatcode(
            cheatcode::<
                'ecdsa_sign_message'
            >(array!['Secp256r1', sk_low, sk_high, msg_hash_low, msg_hash_high].span())
        );

        if *output[0] == 1 {
            core::panic_with_felt252(*output[1]);
        }

        let r = to_u256(*output[1], *output[2]);
        let s = to_u256(*output[3], *output[4]);

        (r, s)
    }
}

impl Secp256r1CurveVerifierImpl of VerifierTrait<
    KeyPair<u256, Secp256r1Point>, u256, (u256, u256)
> {
    fn verify(
        self: KeyPair<u256, Secp256r1Point>, message_hash: u256, signature: (u256, u256)
    ) -> bool {
        let (r, s) = signature;
        is_valid_signature::<Secp256r1Point>(message_hash, r, s, self.public_key)
    }
}
