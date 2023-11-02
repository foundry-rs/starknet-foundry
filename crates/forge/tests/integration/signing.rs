use indoc::indoc;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_passed, test_case};

#[test]
fn simple_signing_flow() {
    let test = test_case!(indoc!(
        r#"
            use snforge_std::signature::{ StarkCurveKeyPair, StarkCurveKeyPairTrait, Signer, Verifier };

            #[test]
            fn test() {
                let mut key_pair = StarkCurveKeyPairTrait::generate();
                let message_hash = 123456;

                let signature = key_pair.sign(message_hash).unwrap();
                assert(key_pair.verify(message_hash, signature), 'Wrong signature');
            }
        "#
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn try_to_sign_max_felt() {
    let test = test_case!(indoc!(
        r#"
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
        "#
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}
