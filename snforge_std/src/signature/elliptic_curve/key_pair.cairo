use starknet::testing::cheatcode;
use super::interface::Signer;
use super::interface::Verifier;
use super::ec::EllipticCurve;

use starknet::syscalls::keccak_syscall;

use snforge_std::PrintTrait;
use array::ArrayTrait;
use array::SpanTrait;
use traits::Into;
use traits::TryInto;
use serde::Serde;
use starknet::secp256_trait::{
    Signature, recover_public_key, Secp256Trait, Secp256PointTrait, signature_from_vrs, is_valid_signature
};
use starknet::secp256k1::{Secp256k1Point, Secp256k1PointImpl, secp256k1_new_syscall};
use starknet::{
    SyscallResult, SyscallResultTrait
};

fn to_u256(low: felt252, high: felt252) -> u256 {
    u256 { low: low.try_into().unwrap(), high: high.try_into().unwrap() }
}

fn from_u256(x: u256) -> (felt252, felt252) {
    (x.low.into(), x.high.into())
}

#[derive(Copy, Drop)]
struct KeyPair {
    private_key: u256,
    public_key: Secp256k1Point,
    curve: EllipticCurve,
}

#[generate_trait]
impl KeyPairImpl of KeyPairTrait {
    fn generate(curve: EllipticCurve) -> KeyPair {
        let mut inputs = array![0];
        curve.serialize(ref inputs);
        
        let output: Span<felt252> = cheatcode::<'generate_ecdsa_keys'>(inputs.span());

        let private_key = to_u256(*output[0], *output[1]);
        let x = to_u256(*output[2], *output[3]);
        let y = to_u256(*output[4], *output[5]);
        
        let public_key = secp256k1_new_syscall(x, y).unwrap_syscall().unwrap();

        KeyPair { private_key, public_key, curve }
    }

    fn from_private(private_key: u256, curve: EllipticCurve) -> KeyPair {
        let mut serialized_curve = array![];
        curve.serialize(ref serialized_curve);

        let (pk_low, pk_high) = from_u256(private_key);

        let output = cheatcode::<'get_public_key'>(array![pk_low, pk_high, *serialized_curve[0]].span());

        let x = to_u256(*output[0], *output[1]);
        let y = to_u256(*output[2], *output[3]);

        let public_key = secp256k1_new_syscall(x, y).unwrap_syscall().unwrap();

        KeyPair { private_key, public_key, curve }
    }
}

impl KeyPairSigner of Signer<KeyPair> {
    fn sign(
        ref self: KeyPair, message_hash: u256
    ) -> Result<(u256, u256), felt252> {
        let mut serialized_curve = array![];
        self.curve.serialize(ref serialized_curve);

        let (pk_low, pk_high) = from_u256(self.private_key);
        let (msg_hash_low, msg_hash_high) = from_u256(message_hash);

        let output = cheatcode::<
            'ecdsa_sign_message'
        >(array![pk_low, pk_high, *serialized_curve[0], msg_hash_low, msg_hash_high].span());

        let r = to_u256(*output[0], *output[1]);
        let s = to_u256(*output[2], *output[3]);

        Result::Ok((r, s))
    }
}

impl KeyPairVerifier of Verifier<KeyPair> {
    fn verify(
        ref self: KeyPair, message_hash: u256, signature: (u256, u256)
    ) -> bool {
        let (r, s) = signature;
        is_valid_signature::<Secp256k1Point>(message_hash, r, s, self.public_key)
    }
}
