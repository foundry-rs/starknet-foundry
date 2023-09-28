use indoc::indoc;
use utils::running_tests::run_test_case;
use utils::{assert_passed, test_case};

#[test]
fn fuzzed_argument() {
    let test = test_case!(indoc!(
        r#"        
        fn adder(a: felt252, b: felt252) -> felt252 {
            a + b
        }

        #[test]
        fn fuzzed_argument(b: felt252) {
            let result = adder(2, b);
            assert(result == 2 + b, '2 + b == 2 + b');
        }
    "#
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn fuzzer_different_types() {
    let test = test_case!(indoc!(
        r#"
        #[test]
        fn uint256_arg(a: u256) {
            if a <= 5_u256 {
                assert(2 == 2, '2 == 2');
            } else {
                let x = a - 5_u256;
                assert(x == a - 5_u256, 'x != a - 5');
            }
        }
    "#
    ));

    let result = run_test_case(&test);

    assert_passed!(result);
}
