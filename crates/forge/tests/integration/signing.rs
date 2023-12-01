use indoc::indoc;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_passed, test_case};

#[test]
fn simple_signing_flow_stark_curve() {
    let test = test_case!(indoc!(
        r"
            use snforge_std::signature::stark_curve::{ StarkCurveKeyPair, StarkCurveKeyPairTrait, Signer, Verifier };

            #[test]
            fn test() {
                let mut key_pair = StarkCurveKeyPairTrait::generate();
                let message_hash = 123456;

                let signature = key_pair.sign(message_hash).unwrap();
                assert(key_pair.verify(message_hash, signature), 'Wrong signature');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn try_to_sign_max_felt() {
    let test = test_case!(indoc!(
        r"
            use snforge_std::signature::stark_curve::{ StarkCurveKeyPair, StarkCurveKeyPairTrait, Signer };

            #[test]
            fn test() {
                let mut key_pair = StarkCurveKeyPairTrait::generate();
                let max_felt = 3618502788666131213697322783095070105623107215331596699973092056135872020480;

                match key_pair.sign(max_felt) {
                    Result::Ok(_) => panic_with_felt252('shouldve panicked'),
                    Result::Err(msg) => {
                        assert(msg == 'message_hash out of range', msg);
                    }
                }
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn test_secp256_k1_curve() {
    let test = test_case!(indoc!(
        r"
            use snforge_std::signature::elliptic_curve::{ KeyPair, KeyPairTrait, Signer, Verifier };
            use starknet::secp256k1::{ Secp256k1Impl, Secp256k1Point, Secp256k1PointImpl };

            #[test]
            fn test() {
                let key_pair = KeyPairTrait::<Secp256k1Point>::generate();
                
                let msg_hash: u256 = 0xbadc0ffee;
                let (r, s) = key_pair.sign(msg_hash);
            
                let is_valid = key_pair.verify(msg_hash, (r, s));
                assert(is_valid, 'Signature should be valid');
            
                let key_pair2 = KeyPairTrait::<Secp256k1Point>::from_private(key_pair.secret_key);
                assert(key_pair.secret_key == key_pair2.secret_key, 'Private keys should be equal');
                assert(key_pair.public_key.get_coordinates() == key_pair2.public_key.get_coordinates(), 'Public keys should be equal');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn test_secp256_r1_curve() {
    let test = test_case!(indoc!(
        r"
            use snforge_std::signature::elliptic_curve::{ KeyPair, KeyPairTrait, Signer, Verifier };
            use starknet::secp256r1::{ Secp256r1Impl, Secp256r1Point, Secp256r1PointImpl };

            #[test]
            fn test() {
                let key_pair = KeyPairTrait::<Secp256r1Point>::generate();
                
                let msg_hash: u256 = 0xbadc0ffee;
                let (r, s) = key_pair.sign(msg_hash);
            
                let is_valid = key_pair.verify(msg_hash, (r, s));
                assert(is_valid, 'Signature should be valid');
            
                let key_pair2 = KeyPairTrait::<Secp256r1Point>::from_private(key_pair.secret_key);
                assert(key_pair.secret_key == key_pair2.secret_key, 'Private keys should be equal');
                assert(key_pair.public_key.get_coordinates() == key_pair2.public_key.get_coordinates(), 'Public keys should be equal');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn test_secp256_curve() {
    let test = test_case!(indoc!(
        r"
            use snforge_std::signature::elliptic_curve::{ KeyPair, KeyPairTrait, Signer, Verifier };
            use starknet::secp256r1::{ Secp256r1Impl, Secp256r1Point, Secp256r1PointImpl };
            use starknet::secp256k1::{ Secp256k1Impl, Secp256k1Point, Secp256k1PointImpl };

            #[test]
            fn test() {
                let secret_key = 554433;

                let key_pair_k1 = KeyPairTrait::<Secp256k1Point>::from_private(secret_key);
                let key_pair_r1 = KeyPairTrait::<Secp256r1Point>::from_private(secret_key);
                
                assert(key_pair_k1.secret_key == key_pair_r1.secret_key, 'Private keys should equal');
                assert(key_pair_k1.public_key.get_coordinates() != key_pair_r1.public_key.get_coordinates(), 'Public keys should be different');

                let msg_hash: u256 = 0xbadc0ffee;

                let sig_k1 = key_pair_k1.sign(msg_hash);
                let sig_r1 = key_pair_r1.sign(msg_hash);
                
                assert(sig_k1 != sig_r1, 'Signatures should be different');

                assert(key_pair_k1.verify(msg_hash, sig_k1) == true, 'Signature should be valid');
                assert(key_pair_r1.verify(msg_hash, sig_r1) == true, 'Signature should be valid');
                
                assert(key_pair_k1.verify(msg_hash, sig_r1) == false, 'Signature should be invalid');
                assert(key_pair_r1.verify(msg_hash, sig_k1) == false, 'Signature should be invalid');
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}
