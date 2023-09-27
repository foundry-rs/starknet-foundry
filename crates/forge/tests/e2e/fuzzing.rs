use crate::e2e::common::runner::{runner, setup_package};
use indoc::indoc;

#[test]
fn fuzzing() {
    let temp = setup_package("fuzzing");
    let snapbox = runner().arg("fuzzing");

    snapbox
        .current_dir(&temp)
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 5 test(s) from fuzzing package
        Running 5 test(s) from src/
        [PASS] fuzzing::tests::adding
        [PASS] fuzzing::tests::fuzzed_argument (fuzzer runs = 256)
        [PASS] fuzzing::tests::fuzzed_both_arguments (fuzzer runs = 256)
        [PASS] fuzzing::tests::passing
        [FAIL] fuzzing::tests::failing_fuzz (fuzzer runs = 1, arguments = [[..], [..]])

        Failure data:
            original value: [593979512822486835600413552099926114], converted to a string: [result == a + b]

        Running 0 test(s) from tests/
        Tests: 4 passed, 1 failed, 0 skipped
        Fuzzer seed: [..]

        Failures:
            fuzzing::tests::failing_fuzz
        "#});
}

#[test]
fn fuzzing_set_runs() {
    let temp = setup_package("fuzzing");
    let snapbox = runner().arg("fuzzing");

    snapbox
        .current_dir(&temp)
        .args(["--fuzzer-runs", "10"])
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 5 test(s) from fuzzing package
        Running 5 test(s) from src/
        [PASS] fuzzing::tests::adding
        [PASS] fuzzing::tests::fuzzed_argument (fuzzer runs = 10)
        [PASS] fuzzing::tests::fuzzed_both_arguments (fuzzer runs = 10)
        [PASS] fuzzing::tests::passing
        [FAIL] fuzzing::tests::failing_fuzz (fuzzer runs = 1, arguments = [[..], [..]])

        Failure data:
            original value: [593979512822486835600413552099926114], converted to a string: [result == a + b]

        Running 0 test(s) from tests/
        Tests: 4 passed, 1 failed, 0 skipped
        Fuzzer seed: [..]

        Failures:
            fuzzing::tests::failing_fuzz
        "#});
}

#[test]
fn fuzzing_set_seed() {
    let temp = setup_package("fuzzing");
    let snapbox = runner().arg("fuzzing");

    snapbox
        .current_dir(&temp)
        .args(["--fuzzer-seed", "1234"])
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 5 test(s) from fuzzing package
        Running 5 test(s) from src/
        [PASS] fuzzing::tests::adding
        [PASS] fuzzing::tests::fuzzed_argument (fuzzer runs = 256)
        [PASS] fuzzing::tests::fuzzed_both_arguments (fuzzer runs = 256)
        [PASS] fuzzing::tests::passing
        [FAIL] fuzzing::tests::failing_fuzz (fuzzer runs = 1, arguments = [341193006617052062735469547906266761132387087565883400330245273769314796067, 527032148587025968655727080236627006840087681556560587927394687076297616261])

        Failure data:
            original value: [593979512822486835600413552099926114], converted to a string: [result == a + b]

        Running 0 test(s) from tests/
        Tests: 4 passed, 1 failed, 0 skipped
        Fuzzer seed: 1234

        Failures:
            fuzzing::tests::failing_fuzz
        "#});
}

#[test]
fn fuzzing_incorrect_runs() {
    let temp = setup_package("fuzzing");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .args(["--fuzzer-runs", "0"])
        .assert()
        .stderr_matches(indoc! {r#"
        error: invalid value '0' for '--fuzzer-runs <FUZZER_RUNS>': Number of fuzzer runs must be greater than or equal to 3

        For more information, try '--help'.
        "#});
}

#[test]
fn fuzzing_incorrect_function_args() {
    let temp = setup_package("fuzzing");
    let snapbox = runner().arg("incorrect_args");

    snapbox
        .current_dir(&temp)
        .assert()
        .code(2)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        
        
        Collected 2 test(s) from fuzzing package
        Running 0 test(s) from src/
        Running 2 test(s) from tests/
        [PASS] tests::incorrect_args::correct_args (fuzzer runs = 256)
        [ERROR] Fuzzer only supports felt252 arguments, and test tests::incorrect_args::incorrect_args defines arguments that are not felt252 type
        "#});
}
