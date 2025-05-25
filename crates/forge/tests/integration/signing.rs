use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use test_utils::running_tests::run_test_case;
use test_utils::{runner::assert_passed, test_case};

#[test]
fn test_stark_sign_msg_hash_range() {
    let test = test_case!(indoc!(
        r"
            use snforge_std::signature::{KeyPairTrait, SignError};
            use snforge_std::signature::stark_curve::{StarkCurveKeyPairImpl, StarkCurveSignerImpl, StarkCurveVerifierImpl};
            
            const UPPER_BOUND: felt252 = 0x800000000000000000000000000000000000000000000000000000000000000;

            #[test]
            fn valid_range() {
                let key_pair = KeyPairTrait::<felt252, felt252>::generate();
                
                let msg_hash = UPPER_BOUND - 1;
                let (r, s): (felt252, felt252) = key_pair.sign(msg_hash).unwrap();
            
                let is_valid = key_pair.verify(msg_hash, (r, s));
                assert(is_valid, 'Signature should be valid');
            }

            #[test]
            fn invalid_range() {
                let key_pair = KeyPairTrait::<felt252, felt252>::generate();
                
                // message_hash should be smaller than UPPER_BOUND
                // https://github.com/starkware-libs/crypto-cpp/blob/78e3ed8dc7a0901fe6d62f4e99becc6e7936adfd/src/starkware/crypto/ecdsa.cc#L65
                let msg_hash = UPPER_BOUND;
                assert(key_pair.sign(msg_hash).unwrap_err() == SignError::HashOutOfRange, '');
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn test_stark_curve() {
    let test = test_case!(indoc!(
        r"
        use snforge_std::signature::KeyPairTrait;
        use snforge_std::signature::stark_curve::{StarkCurveKeyPairImpl, StarkCurveSignerImpl, StarkCurveVerifierImpl};
        
        #[test]
        fn simple_signing_flow() {
            let key_pair = KeyPairTrait::<felt252, felt252>::generate();
            
            let msg_hash = 0xbadc0ffee;
            let (r, s): (felt252, felt252) = key_pair.sign(msg_hash).unwrap();
        
            let is_valid = key_pair.verify(msg_hash, (r, s));
            assert(is_valid, 'Signature should be valid');
        
            let key_pair2 = KeyPairTrait::<felt252, felt252>::from_secret_key(key_pair.secret_key);
            assert(key_pair.secret_key == key_pair2.secret_key, 'Secret keys should be equal');
            assert(key_pair.public_key == key_pair2.public_key, 'Public keys should be equal');
        }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn test_secp256k1_curve() {
    let test = test_case!(indoc!(
        r"
            use snforge_std::signature::KeyPairTrait;
            use snforge_std::signature::secp256k1_curve::{Secp256k1CurveKeyPairImpl, Secp256k1CurveSignerImpl, Secp256k1CurveVerifierImpl};
            use starknet::secp256k1::{Secp256k1Point, Secp256k1PointImpl};
            use core::starknet::SyscallResultTrait;

            #[test]
            fn simple_signing_flow() {
                let key_pair = KeyPairTrait::<u256, Secp256k1Point>::generate();
                
                let msg_hash = 0xbadc0ffee;
                let (r, s): (u256, u256) = key_pair.sign(msg_hash).unwrap();
            
                let is_valid = key_pair.verify(msg_hash, (r, s));
                assert(is_valid, 'Signature should be valid');
            
                let key_pair2 = KeyPairTrait::<u256, Secp256k1Point>::from_secret_key(key_pair.secret_key);
                assert(key_pair.secret_key == key_pair2.secret_key, 'Secret keys should be equal');
                assert(key_pair.public_key.get_coordinates().unwrap_syscall() == key_pair2.public_key.get_coordinates().unwrap_syscall(), 'Public keys should be equal');
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn test_secp256r1_curve() {
    let test = test_case!(indoc!(
        r"
            use snforge_std::signature::KeyPairTrait;
            use snforge_std::signature::secp256r1_curve::{Secp256r1CurveKeyPairImpl, Secp256r1CurveSignerImpl, Secp256r1CurveVerifierImpl};
            use starknet::secp256r1::{Secp256r1Point, Secp256r1PointImpl};
            use core::starknet::SyscallResultTrait;

            #[test]
            fn simple_signing_flow() {
                let key_pair = KeyPairTrait::<u256, Secp256r1Point>::generate();
                
                let msg_hash = 0xbadc0ffee;
                let (r, s): (u256, u256) = key_pair.sign(msg_hash).unwrap();
            
                let is_valid = key_pair.verify(msg_hash, (r, s));
                assert(is_valid, 'Signature should be valid');
            
                let key_pair2 = KeyPairTrait::<u256, Secp256r1Point>::from_secret_key(key_pair.secret_key);
                assert(key_pair.secret_key == key_pair2.secret_key, 'Secret keys should be equal');
                assert(key_pair.public_key.get_coordinates().unwrap_syscall() == key_pair2.public_key.get_coordinates().unwrap_syscall(), 'Public keys should be equal');
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn test_secp256_curves() {
    let test = test_case!(indoc!(
        r"
            use snforge_std::signature::KeyPairTrait;
            use snforge_std::signature::secp256k1_curve::{Secp256k1CurveKeyPairImpl, Secp256k1CurveSignerImpl, Secp256k1CurveVerifierImpl};
            use snforge_std::signature::secp256r1_curve::{Secp256r1CurveKeyPairImpl, Secp256r1CurveSignerImpl, Secp256r1CurveVerifierImpl};
            use starknet::secp256k1::{Secp256k1Point, Secp256k1PointImpl};
            use starknet::secp256r1::{Secp256r1Point, Secp256r1PointImpl};
            use core::starknet::SyscallResultTrait;

            #[test]
            fn secp256_curves() {
                let secret_key = 554433;

                let key_pair_k1 = KeyPairTrait::<u256, Secp256k1Point>::from_secret_key(secret_key);
                let key_pair_r1 = KeyPairTrait::<u256, Secp256r1Point>::from_secret_key(secret_key);
                
                assert(key_pair_k1.secret_key == key_pair_r1.secret_key, 'Secret keys should equal');
                assert(key_pair_k1.public_key.get_coordinates().unwrap_syscall() != key_pair_r1.public_key.get_coordinates().unwrap_syscall(), 'Public keys should be different');

                let msg_hash: u256 = 0xbadc0ffee;

                let sig_k1 = key_pair_k1.sign(msg_hash).unwrap();
                let sig_r1 = key_pair_r1.sign(msg_hash).unwrap();
                
                assert(sig_k1 != sig_r1, 'Signatures should be different');

                assert(key_pair_k1.verify(msg_hash, sig_k1) == true, 'Signature should be valid');
                assert(key_pair_r1.verify(msg_hash, sig_r1) == true, 'Signature should be valid');
                
                assert(key_pair_k1.verify(msg_hash, sig_r1) == false, 'Signature should be invalid');
                assert(key_pair_r1.verify(msg_hash, sig_k1) == false, 'Signature should be invalid');
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn test_stark_secp256k1_curves() {
    let test = test_case!(indoc!(
        r"
            use snforge_std::signature::KeyPairTrait;
            use snforge_std::signature::stark_curve::{StarkCurveKeyPairImpl, StarkCurveSignerImpl, StarkCurveVerifierImpl};
            use snforge_std::signature::secp256k1_curve::{Secp256k1CurveKeyPairImpl, Secp256k1CurveSignerImpl, Secp256k1CurveVerifierImpl};
            use starknet::secp256k1::{Secp256k1Point, Secp256k1PointImpl};
            use core::starknet::SyscallResultTrait;
            
            #[test]
            fn stark_secp256k1_curves() {
                let secret_key = 554433;
            
                let key_pair_stark = KeyPairTrait::<felt252, felt252>::from_secret_key(secret_key);
                let key_pair_secp256k1 = KeyPairTrait::<u256, Secp256k1Point>::from_secret_key(secret_key.into());
                
                assert(key_pair_stark.secret_key.into() == key_pair_secp256k1.secret_key, 'Secret keys should equal');
            
                let (pk_x_secp256k1, _pk_y_secp256k1) = key_pair_secp256k1.public_key.get_coordinates().unwrap_syscall();
            
                assert(key_pair_stark.public_key.into() != pk_x_secp256k1, 'Public keys should be different');
            
                let msg_hash = 0xbadc0ffee;
            
                let (r_stark, s_stark): (felt252, felt252) = key_pair_stark.sign(msg_hash).unwrap();
                let (r_secp256k1, s_secp256k1): (u256, u256) = key_pair_secp256k1.sign(msg_hash.into()).unwrap();
                
                assert(r_stark.into() != r_secp256k1, 'Signatures should be different');
                assert(s_stark.into() != s_secp256k1, 'Signatures should be different');
            
                assert(key_pair_stark.verify(msg_hash, (r_stark, s_stark)) == true, 'Signature should be valid');
                assert(key_pair_secp256k1.verify(msg_hash.into(), (r_secp256k1, s_secp256k1)) == true, 'Signature should be valid');
                
                assert(key_pair_secp256k1.verify(msg_hash.into(), (r_stark.into(), s_stark.into())) == false, 'Signature should be invalid');
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn test_invalid_secret_key() {
    let test = test_case!(indoc!(
        r"
            use snforge_std::signature::{KeyPair, KeyPairTrait, SignError};
            use snforge_std::signature::stark_curve::{StarkCurveKeyPairImpl, StarkCurveSignerImpl};
            use snforge_std::signature::secp256k1_curve::{Secp256k1CurveKeyPairImpl, Secp256k1CurveSignerImpl};
            use snforge_std::signature::secp256r1_curve::{Secp256r1CurveKeyPairImpl, Secp256r1CurveSignerImpl};
            use starknet::secp256k1::{Secp256k1Impl, Secp256k1Point};
            use starknet::secp256r1::{Secp256r1Impl, Secp256r1Point};
            
            #[test]
            #[should_panic(expected: ('invalid secret_key', ))]
            fn from_secret_key_stark() {
                KeyPairTrait::<felt252, felt252>::from_secret_key(0);
            }
            
            #[test]
            #[should_panic(expected: ('invalid secret_key', ))]
            fn from_secret_key_secp256k1() {
                KeyPairTrait::<u256, Secp256k1Point>::from_secret_key(0);
            }
            
            #[test]
            #[should_panic(expected: ('invalid secret_key', ))]
            fn from_secret_key_secp256r1() {
                KeyPairTrait::<u256, Secp256r1Point>::from_secret_key(0);
            }
            
            #[test]
            fn sign_stark() {
                let key_pair = KeyPair { secret_key: 0, public_key: 0x321 } ;
                assert(key_pair.sign(123).unwrap_err() == SignError::InvalidSecretKey, '');
            }
            
            #[test]
            fn sign_secp256k1() {
                let generator = Secp256k1Impl::get_generator_point();
                let key_pair = KeyPair { secret_key: 0, public_key: generator } ;
                assert(key_pair.sign(123).unwrap_err() == SignError::InvalidSecretKey, '');
            }
            
            #[test]
            fn sign_secp256r1() {
                let generator = Secp256r1Impl::get_generator_point();
                let key_pair = KeyPair { secret_key: 0, public_key: generator } ;
                assert(key_pair.sign(123).unwrap_err() == SignError::InvalidSecretKey, '');
            }
        "
    ));

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}
