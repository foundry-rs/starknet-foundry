use forge_runner::test_case_summary::{AnyTestCaseSummary, TestCaseSummary};
use indoc::indoc;
use test_utils::runner::{assert_passed, TestCase};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn fuzzed_argument() {
    let test = test_case!(indoc!(
        r"
        fn adder(a: felt252, b: felt252) -> felt252 {
            a + b
        }

        #[test]
        #[fuzzer]
        fn fuzzed_argument(b: felt252) {
            let result = adder(2, b);
            assert(result == 2 + b, '2 + b == 2 + b');
        }
    "
    ));

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn fuzzer_different_types() {
    let test = test_case!(indoc!(
        r"
        #[test]
        #[fuzzer]
        fn fuzzer_different_types(a: u256) {
            if a <= 5_u256 {
                assert(2 == 2, '2 == 2');
            } else {
                let x = a - 5_u256;
                assert(x == a - 5_u256, 'x != a - 5');
            }
        }
    "
    ));

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn fuzzed_while_loop() {
    let test = test_case!(indoc!(
        r"
        #[test]
        #[fuzzer(runs: 256, seed: 100)]
        fn fuzzed_while_loop(a: u8) {
            let mut i: u8 = 0;
            while i != a {
                i += 1;
            };

            assert(1 == 1, '');
        }
    "
    ));

    let result = run_test_case(&test);

    let test_target_summary = TestCase::find_test_result(&result);
    let AnyTestCaseSummary::Fuzzing(TestCaseSummary::Passed { gas_info, .. }) =
        &test_target_summary.test_case_summaries[0]
    else {
        panic!()
    };

    assert_eq!(gas_info.min, 2);
    assert_eq!(gas_info.max, 22);
    assert!((gas_info.mean - 12.).abs() < f64::EPSILON);
    assert!((gas_info.std_deviation - 6.22).abs() < 0.01);
}

#[test]
fn builtin_test() {
    let test = test_case!(indoc!(
        r"
        use core::{
            pedersen::Pedersen, RangeCheck, integer::Bitwise, ec::EcOp, poseidon::Poseidon,
            SegmentArena, gas::GasBuiltin, starknet::System
        };
        use core::circuit::{RangeCheck96, AddMod, MulMod};

        #[test]
        fn test_builtins() {
            core::internal::require_implicit::<Pedersen>();
            core::internal::require_implicit::<RangeCheck>();
            core::internal::require_implicit::<Bitwise>();
            core::internal::require_implicit::<EcOp>();
            core::internal::require_implicit::<Poseidon>();
            core::internal::require_implicit::<SegmentArena>();
            core::internal::require_implicit::<GasBuiltin>();
            core::internal::require_implicit::<System>();
            core::internal::require_implicit::<RangeCheck96>();
            core::internal::require_implicit::<AddMod>();
            core::internal::require_implicit::<MulMod>();
        }
        "
    ));

    let result = run_test_case(&test);

    assert_passed(&result);
}
