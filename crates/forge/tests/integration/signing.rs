use indoc::indoc;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_passed, test_case};

// #[should_panic(expected: ('Currently only Secp256k1 and', 'Secp256r1 curves are supported'))]

// key_pair.secret_key.print();
// let pk = key_pair.public_key.get_coordinates().unwrap_syscall();
// let (x, y) = pk;
// x.print();
// y.print();

#[test]
fn test_key_pair() {
    let test = test_case!(indoc!(
        r"
            use snforge_std::signature::elliptic_curve::{ KeyPair, KeyPairTrait };
            use starknet::secp256k1::{ Secp256k1Impl, Secp256k1Point, Secp256k1PointImpl };
            use starknet::secp256r1::{ Secp256r1Impl, Secp256r1Point, Secp256r1PointImpl };

            #[test]
            fn test_u256_k1() {
                let key_pair = KeyPairTrait::<u256, Secp256k1Point>::generate();
            }

            #[test]
            fn test_u256_r1() {
                let key_pair = KeyPairTrait::<u256, Secp256k1Point>::generate();
            }

            #[test]
            fn test_u256_felt252() {
                let key_pair = KeyPairTrait::<u256, felt252>::generate();
            }

            #[test]
            fn test_felt252_u256() {
                let key_pair = KeyPairTrait::<felt252, u256>::generate();
            }

            #[test]
            fn test_felt252_felt252() {
                let key_pair = KeyPairTrait::<felt252, felt252>::generate();
            }

            #[test]
            fn test_u256_u256() {
                let key_pair = KeyPairTrait::<u256, u256>::generate();
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}

// #[test]
// fn test_invalid_key_pair() {
//     let test = test_case!(indoc!(
//         r"
//             use snforge_std::signature::elliptic_curve::{ KeyPair, KeyPairTrait, Signer, Verifier };
//             use starknet::secp256k1::{ Secp256k1Impl, Secp256k1Point, Secp256k1PointImpl };
//             use starknet::secp256r1::{ Secp256r1Impl, Secp256r1Point, Secp256r1PointImpl };

//             #[test]
//             #[should_panic]
//             fn test_u256_r1() {
//                 let key_pair = KeyPairTrait::<Secp256k1Point, Secp256k1Point>::generate();
//             }

//             #[test]
//             #[should_panic]
//             fn test_u256_r1() {
//                 let key_pair = KeyPairTrait::<felt252, Secp256k1Point>::generate();
//             }

//             #[test]
//             #[should_panic]
//             fn test_u128_felt252() {
//                 let key_pair = KeyPairTrait::<u128, felt252>::generate();
//             }

//             #[test]
//             #[should_panic]
//             fn test_felt252_u128() {
//                 let key_pair = KeyPairTrait::<felt252, u128>::generate();
//             }
//         "
//     ));

//     let result = run_test_case(&test);

//     assert_passed!(result);
// }

#[test]
fn simple_signing_flow_stark_curve() {
    let test = test_case!(indoc!(
        r"
        use snforge_std::signature::elliptic_curve::interface::{ KeyPair, KeyPairTrait, Signer, Verifier };
        use snforge_std::signature::elliptic_curve::stark_curve::{ StarkCurveKeyPairImpl, StarkCurveSignerImpl, StarkCurveVerifierImpl };
        use ec::{ EcPointTrait, NonZeroEcPoint };

        #[test]
        fn test() {
            // let mut key_pair = KeyPairTrait::<felt252, felt252>::generate();
            // let message_hash = 123456;

            // let signature = key_pair.sign(message_hash);
            // assert(key_pair.verify(message_hash, signature), 'Wrong signature');

            let mut key_pair = KeyPairTrait::<felt252, NonZeroEcPoint>::generate();
            let message_hash = 123456;

            let signature = key_pair.sign(message_hash);
            assert(key_pair.verify(message_hash, signature), 'Wrong signature');

            let key_pair2 = KeyPairTrait::<felt252, NonZeroEcPoint>::from_secret_key(key_pair.secret_key);
            assert(key_pair.secret_key == key_pair2.secret_key, 'Secret keys should be equal');
            assert(key_pair.public_key.coordinates() == key_pair2.public_key.coordinates(), 'Public keys should be equal');
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
            use snforge_std::signature::{ StarkCurveKeyPair, StarkCurveKeyPairTrait, Signer };

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
            
                let key_pair2 = KeyPairTrait::<Secp256k1Point>::from_secret_key(key_pair.secret_key);
                assert(key_pair.secret_key == key_pair2.secret_key, 'Secret keys should be equal');
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
            
                let key_pair2 = KeyPairTrait::<Secp256r1Point>::from_secret_key(key_pair.secret_key);
                assert(key_pair.secret_key == key_pair2.secret_key, 'Secret keys should be equal');
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

                let key_pair_k1 = KeyPairTrait::<Secp256k1Point>::from_secret_key(secret_key);
                let key_pair_r1 = KeyPairTrait::<Secp256r1Point>::from_secret_key(secret_key);
                
                assert(key_pair_k1.secret_key == key_pair_r1.secret_key, 'Secret keys should equal');
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

#[test]
fn test_unsupported_curve() {
    let test = test_case!(indoc!(
        r"
            use snforge_std::signature::elliptic_curve::{ KeyPair, KeyPairTrait, Signer, Verifier };
            use starknet::secp256_trait::{ Secp256Trait, Secp256PointTrait };
            use starknet::secp256k1::{ Secp256k1Impl, Secp256k1Point, Secp256k1PointImpl, secp256k1_new_syscall };
            use starknet::{ SyscallResult, SyscallResultTrait };

            #[derive(Copy, Drop)]
            struct UnsupportedCurvePoint {
                inner: Secp256k1Point
            }
            
            impl UnsupportedCurveImpl of Secp256Trait<UnsupportedCurvePoint> {
                fn get_curve_size() -> u256 {
                    0x123.into()
                }
                fn get_generator_point() -> UnsupportedCurvePoint {
                    UnsupportedCurvePoint { inner: Secp256k1Impl::get_generator_point() }
                }
            
                fn secp256_ec_new_syscall(x: u256, y: u256) -> SyscallResult<Option<UnsupportedCurvePoint>> {
                    let point = UnsupportedCurvePoint { inner: Secp256k1Impl::get_generator_point() };
                    SyscallResult::Ok(Option::Some(point))
                }
                fn secp256_ec_get_point_from_x_syscall(
                    x: u256, y_parity: bool
                ) -> SyscallResult<Option<UnsupportedCurvePoint>> {
                    let point = UnsupportedCurvePoint { inner: Secp256k1Impl::get_generator_point() };
                    SyscallResult::Ok(Option::Some(point))
                }
            }
            
            impl UnsupportedCurvePointImpl of Secp256PointTrait<UnsupportedCurvePoint> {
                fn get_coordinates(self: UnsupportedCurvePoint) -> SyscallResult<(u256, u256)> {
                    SyscallResult::Ok((1.into(), 1.into()))
                }
                fn add(self: UnsupportedCurvePoint, other: UnsupportedCurvePoint) -> SyscallResult<UnsupportedCurvePoint> {
                    SyscallResult::Ok(self)
                }
                fn mul(self: UnsupportedCurvePoint, scalar: u256) -> SyscallResult<UnsupportedCurvePoint> {
                    SyscallResult::Ok(self)
                }
            }
            
            #[test]
            #[should_panic(expected: ('Currently only Secp256k1 and', 'Secp256r1 curves are supported'))]
            fn test() {
                let key_pair = KeyPairTrait::<UnsupportedCurvePoint>::generate();
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn test_invalid_secret_key_secp256() {
    let test = test_case!(indoc!(
        r"
            use snforge_std::signature::elliptic_curve::{ KeyPair, KeyPairTrait, Signer, Verifier };
            use starknet::secp256r1::{ Secp256r1Impl, Secp256r1Point, Secp256r1PointImpl };
            use starknet::secp256k1::{ Secp256k1Impl, Secp256k1Point, Secp256k1PointImpl };

            #[test]
            #[should_panic(expected: ('invalid secret_key', ))]
            fn test_from_secret_key_secp256k1() {
                let key_pair = KeyPairTrait::<Secp256k1Point>::from_secret_key(0.into());
            }

            #[test]
            #[should_panic(expected: ('invalid secret_key', ))]
            fn test_from_secret_key_secp256r1() {
                let key_pair = KeyPairTrait::<Secp256r1Point>::from_secret_key(0.into());
            }

            #[test]
            #[should_panic(expected: ('invalid secret_key', ))]
            fn test_sign_secp256k1() {
                let key_pair = KeyPair { secret_key: 0.into(), public_key: Secp256k1Impl::get_generator_point() } ;
                let (r, s) = key_pair.sign(123.into());
            }

            #[test]
            #[should_panic(expected: ('invalid secret_key', ))]
            fn test_sign_secp256r1() {
                let key_pair = KeyPair { secret_key: 0.into(), public_key: Secp256r1Impl::get_generator_point() } ;
                let (r, s) = key_pair.sign(123.into());
            }
        "
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}
