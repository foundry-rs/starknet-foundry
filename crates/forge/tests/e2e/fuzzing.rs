use super::common::runner::{setup_package, test_runner};
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};

#[test]
fn fuzzing() {
    let temp = setup_package("fuzzing");

    let output = test_runner(&temp).arg("fuzzing::").assert().code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 13 test(s) from fuzzing package
        Running 13 test(s) from src/
        [PASS] fuzzing::tests::adding [..]
        [PASS] fuzzing::tests::fuzzed_argument (runs: 256, [..]
        [PASS] fuzzing::tests::fuzzed_both_arguments (runs: 256, [..]
        [PASS] fuzzing::tests::passing [..]
        [FAIL] fuzzing::tests::failing_fuzz (runs: 1, arguments: [[..], [..]])

        Failure data:
            0x726573756c74203d3d2061202b2062 ('result == a + b')

        [PASS] fuzzing::tests::custom_fuzzer_config (runs: 10, [..]
        [PASS] fuzzing::tests::uint8_arg (runs: 256, [..]
        [PASS] fuzzing::tests::fuzzed_while_loop (runs: 256, gas: {max: ~[..], min: ~[..], mean: ~[..], std deviation: ~[..]})
        [PASS] fuzzing::tests::uint16_arg (runs: 256, [..]
        [PASS] fuzzing::tests::uint32_arg (runs: 256, [..]
        [PASS] fuzzing::tests::uint64_arg (runs: 256, [..]
        [PASS] fuzzing::tests::uint128_arg (runs: 256, [..]
        [PASS] fuzzing::tests::uint256_arg (runs: 256, [..]
        Running 0 test(s) from tests/
        Tests: 12 passed, 1 failed, 0 skipped, 0 ignored, 11 filtered out
        Fuzzer seed: [..]

        Failures:
            fuzzing::tests::failing_fuzz
        "},
    );
}

#[test]
fn fuzzing_set_runs() {
    let temp = setup_package("fuzzing");

    let output = test_runner(&temp)
        .args(["fuzzing::", "--fuzzer-runs", "10"])
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 13 test(s) from fuzzing package
        Running 13 test(s) from src/
        [PASS] fuzzing::tests::adding [..]
        [PASS] fuzzing::tests::fuzzed_argument (runs: 10, [..]
        [PASS] fuzzing::tests::fuzzed_both_arguments (runs: 10, [..]
        [PASS] fuzzing::tests::passing [..]
        [FAIL] fuzzing::tests::failing_fuzz (runs: 1, arguments: [[..], [..]])

        Failure data:
            0x726573756c74203d3d2061202b2062 ('result == a + b')

        [PASS] fuzzing::tests::custom_fuzzer_config (runs: 10, [..]
        [PASS] fuzzing::tests::uint8_arg (runs: 10, [..]
        [PASS] fuzzing::tests::fuzzed_while_loop (runs: 256, [..]
        [PASS] fuzzing::tests::uint16_arg (runs: 10, [..]
        [PASS] fuzzing::tests::uint32_arg (runs: 10, [..]
        [PASS] fuzzing::tests::uint64_arg (runs: 10, [..]
        [PASS] fuzzing::tests::uint128_arg (runs: 10, [..]
        [PASS] fuzzing::tests::uint256_arg (runs: 10, [..]
        Running 0 test(s) from tests/
        Tests: 12 passed, 1 failed, 0 skipped, 0 ignored, 11 filtered out
        Fuzzer seed: [..]

        Failures:
            fuzzing::tests::failing_fuzz
        "},
    );
}

#[test]
fn fuzzing_set_seed() {
    let temp = setup_package("fuzzing");

    let output = test_runner(&temp)
        .args(["fuzzing::", "--fuzzer-seed", "1234"])
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 13 test(s) from fuzzing package
        Running 13 test(s) from src/
        [PASS] fuzzing::tests::adding [..]
        [PASS] fuzzing::tests::fuzzed_argument (runs: 256, [..]
        [PASS] fuzzing::tests::fuzzed_both_arguments (runs: 256, [..]
        [PASS] fuzzing::tests::passing [..]
        [FAIL] fuzzing::tests::failing_fuzz (runs: 1, arguments: [[..], [..]])

        Failure data:
            0x726573756c74203d3d2061202b2062 ('result == a + b')

        [PASS] fuzzing::tests::custom_fuzzer_config (runs: 10, [..]
        [PASS] fuzzing::tests::uint8_arg (runs: 256, [..]
        [PASS] fuzzing::tests::fuzzed_while_loop (runs: 256, [..]
        [PASS] fuzzing::tests::uint16_arg (runs: 256, [..]
        [PASS] fuzzing::tests::uint32_arg (runs: 256, [..]
        [PASS] fuzzing::tests::uint64_arg (runs: 256, [..]
        [PASS] fuzzing::tests::uint128_arg (runs: 256, [..]
        [PASS] fuzzing::tests::uint256_arg (runs: 256, [..]
        Running 0 test(s) from tests/
        Tests: 12 passed, 1 failed, 0 skipped, 0 ignored, 11 filtered out
        Fuzzer seed: 1234

        Failures:
            fuzzing::tests::failing_fuzz
        "},
    );
}

#[test]
fn fuzzing_incorrect_runs() {
    let temp = setup_package("fuzzing");

    let output = test_runner(&temp)
        .args(["fuzzing::", "--fuzzer-runs", "0"])
        .assert()
        .code(2);

    assert_stderr_contains(
        output,
        indoc! {r"
        error: invalid value '0' for '--fuzzer-runs <FUZZER_RUNS>': number would be zero for non-zero type

        For more information, try '--help'.
        "},
    );
}

#[test]
fn fuzzing_incorrect_function_args() {
    let temp = setup_package("fuzzing");

    let output = test_runner(&temp)
        .args(["incorrect_args", "--features", "unimplemented"])
        .assert()
        .code(2);

    assert_stdout_contains(
        output,
        indoc! {r"
        error: Trait has no implementation in context: snforge_std::fuzzable::Fuzzable::<fuzzing_integrationtest::incorrect_args::MyStruct, fuzzing_integrationtest::incorrect_args::MyStructDebug>.

        [ERROR] Failed to build test artifacts with Scarb: `scarb` exited with error
        "},
    );
}

#[test]
fn fuzzing_exit_first() {
    let temp = setup_package("fuzzing");

    let output = test_runner(&temp)
        .args(["exit_first_fuzz", "-x"])
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 2 test(s) from fuzzing package
        Running 2 test(s) from tests/
        [FAIL] fuzzing_integrationtest::exit_first_fuzz::exit_first_fails_test (runs: 1, arguments: [[..]])

        Failure data:
            0x32202b2062203d3d2032202b2062 ('2 + b == 2 + b')

        Tests: 0 passed, 1 failed, 1 skipped, 0 ignored, 22 filtered out

        Fuzzer seed: [..]
        Failures:
            fuzzing_integrationtest::exit_first_fuzz::exit_first_fails_test
        "},
    );
}

#[test]
fn fuzzing_exit_first_single_fail() {
    let temp = setup_package("fuzzing");

    let output = test_runner(&temp)
        .args(["exit_first_single_fail", "-x"])
        .assert()
        .code(1);

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 2 test(s) from fuzzing package
        Running 2 test(s) from tests/
        [FAIL] fuzzing_integrationtest::exit_first_single_fail::exit_first_fails_test

        Failure data:
            0x32202b2062203d3d2032202b2062 ('2 + b == 2 + b')

        Failures:
            fuzzing_integrationtest::exit_first_single_fail::exit_first_fails_test

        Tests: 0 passed, 1 failed, 1 skipped, 0 ignored, 22 filtered out
        "},
    );
}

#[test]
fn fuzzing_multiple_attributes() {
    let temp = setup_package("fuzzing");

    let output = test_runner(&temp)
        .arg("multiple_attributes")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 4 test(s) from fuzzing package
        Running 4 test(s) from tests/
        [IGNORE] fuzzing_integrationtest::multiple_attributes::ignored
        [PASS] fuzzing_integrationtest::multiple_attributes::with_should_panic (runs: 256, [..])
        [PASS] fuzzing_integrationtest::multiple_attributes::with_available_gas (runs: 50, [..])
        [PASS] fuzzing_integrationtest::multiple_attributes::with_both (runs: 300, [..])
        Tests: 3 passed, 0 failed, 0 skipped, 1 ignored, 20 filtered out
        "},
    );
}

#[test]
fn generate_arg_cheatcode() {
    let temp = setup_package("fuzzing");

    let output = test_runner(&temp).arg("generate_arg").assert().code(1);

    assert_stdout_contains(
        output,
        indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]


        Collected 2 test(s) from fuzzing package
        Running 2 test(s) from tests/
        [FAIL] fuzzing_integrationtest::generate_arg::generate_arg_incorrect_range

        Failure data:
            "`generate_arg` cheatcode: `min_value` must be <= `max_value`, provided values after deserialization: 101 and 100"

        [PASS] fuzzing_integrationtest::generate_arg::use_generate_arg_outside_fuzzer (gas: ~1)
        Tests: 1 passed, 1 failed, 0 skipped, 0 ignored, 22 filtered out
        "#},
    );
}
