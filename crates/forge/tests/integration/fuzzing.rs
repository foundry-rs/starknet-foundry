use crate::integration::common::running_tests::run_test_case;
use crate::{assert_passed, test_case};
use indoc::indoc;

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
        fn uint8_arg(a: u8) {
            if a == 0_u8 {
                assert(2 == 2, '2 == 2');
            } else {
                let x = a - 5_u8;
                assert(x == a - 5_u8, 'x != a - 5');
            }
        }

        #[test]
        fn uint16_arg(a: u16) {
            if a == 0_u16 {
                assert(2 == 2, '2 == 2');
            } else {
                let x = a - 5_u16;
                assert(x == a - 5_u16, 'x != a - 5');
            }
        }

        #[test]
        fn uint32_arg(a: u32) {
            if a == 0_u32 {
                assert(2 == 2, '2 == 2');
            } else {
                let x = a - 5_u32;
                assert(x == a - 5_u32, 'x != a - 5');
            }
        }

        #[test]
        fn uint64_arg(a: u64) {
            if a == 0_u64 {
                assert(2 == 2, '2 == 2');
            } else {
                let x = a - 5_u64;
                assert(x == a - 5_u64, 'x != a - 5');
            }
        }

        #[test]
        fn uint128_arg(a: u128) {
            if a == 0_u128 {
                assert(2 == 2, '2 == 2');
            } else {
                let x = a - 5_u128;
                assert(x == a - 5_u128, 'x != a - 5');
            }
        }

        #[test]
        fn uint256_arg(a: u256) {
            if a == 0_u256 {
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
