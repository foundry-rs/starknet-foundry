use crate::e2e::common::runner::{runner, setup_package};
use indoc::indoc;

#[test]
fn fuzzing() {
    let temp = setup_package("fuzzing");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 5 test(s) and 1 test file(s)
        Running 5 test(s) from fuzzing package
        [PASS] fuzzing::tests::adding
        [PASS] fuzzing::tests::fuzzed_argument (fuzzer runs = 256)
        [PASS] fuzzing::tests::fuzzed_both_arguments (fuzzer runs = 256)
        [PASS] fuzzing::tests::passing
        [FAIL] fuzzing::tests::failing_fuzz (fuzzer runs = 1, arguments = [[..], [..]])

        Failure data:
            original value: [593979512822486835600413552099926114], converted to a string: [result == a + b]

        Tests: 4 passed, 1 failed, 0 skipped
        Fuzzer seed: [..]

        Failures:
            fuzzing::tests::failing_fuzz
        "#});
}

#[test]
fn fuzzing_set_runs() {
    let temp = setup_package("fuzzing");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .args(["--fuzzer-runs", "10"])
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 5 test(s) and 1 test file(s)
        Running 5 test(s) from fuzzing package
        [PASS] fuzzing::tests::adding
        [PASS] fuzzing::tests::fuzzed_argument (fuzzer runs = 10)
        [PASS] fuzzing::tests::fuzzed_both_arguments (fuzzer runs = 10)
        [PASS] fuzzing::tests::passing
        [FAIL] fuzzing::tests::failing_fuzz (fuzzer runs = 1, arguments = [[..], [..]])

        Failure data:
            original value: [593979512822486835600413552099926114], converted to a string: [result == a + b]

        Tests: 4 passed, 1 failed, 0 skipped
        Fuzzer seed: [..]

        Failures:
            fuzzing::tests::failing_fuzz
        "#});
}

#[test]
fn fuzzing_set_seed() {
    let temp = setup_package("fuzzing");
    let snapbox = runner();

    snapbox
        .current_dir(&temp)
        .args(["--fuzzer-seed", "1234"])
        .assert()
        .code(1)
        .stdout_matches(indoc! {r#"
        [..]Compiling[..]
        [..]Finished[..]
        Collected 5 test(s) and 1 test file(s)
        Running 5 test(s) from fuzzing package
        [PASS] fuzzing::tests::adding
        [PASS] fuzzing::tests::fuzzed_argument (fuzzer runs = 256)
        [PASS] fuzzing::tests::fuzzed_both_arguments (fuzzer runs = 256)
        [PASS] fuzzing::tests::passing
        [FAIL] fuzzing::tests::failing_fuzz (fuzzer runs = 1, arguments = [2176913491924129583795008547758153887527303407267805286823327310126940911830, 2350787482052932408706155311404918127641790603096372587257662840043562009418])

        Failure data:
            original value: [593979512822486835600413552099926114], converted to a string: [result == a + b]

        Tests: 4 passed, 1 failed, 0 skipped
        Fuzzer seed: [..]

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
        error: invalid value '0' for '--fuzzer-runs <FUZZER_RUNS>': Number of fuzzer runs must be greater than or equal to 1

        For more information, try '--help'.
        "#});
}
