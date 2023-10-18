use crate::integration::common::running_tests::run_test_case;
use crate::{assert_passed, test_case};
use indoc::indoc;

#[test]
fn generate_keys_sing_verify() {
    let test = test_case!(indoc!(
        r#"
            use snforge_std::{ StarkCurveKeyPair, StarkCurveKeyPairTrait };

            #[test]
            fn test() {
                let mut key_pair = StarkCurveKeyPairTrait::generate();
                let message_hash = 123456;

                let signature = key_pair.sign(message_hash);
                assert(key_pair.verify(message_hash, signature), 'Wrong signature');
            }
        "#
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn try_to_sign_max_felt_plus_1() {
    let test = test_case!(indoc!(
        r#"
            use snforge_std::{ StarkCurveKeyPair, StarkCurveKeyPairTrait };

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
