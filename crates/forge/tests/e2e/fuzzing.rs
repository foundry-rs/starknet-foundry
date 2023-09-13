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
        Running fuzzer for fuzzing::tests::fuzzed_argument, 256 runs:
        [PASS] fuzzing::tests::fuzzed_argument
        Running fuzzer for fuzzing::tests::fuzzed_both_arguments, 256 runs:
        [PASS] fuzzing::tests::fuzzed_both_arguments
        [PASS] fuzzing::tests::passing
        Running fuzzer for fuzzing::tests::failing_fuzz, 256 runs:
        [FAIL] fuzzing::tests::failing_fuzz

        Failure data:
            original value: [593979512822486835600413552099926114], converted to a string: [result == a + b]

        Fuzz test failed on argument(s) [[..], [..]] after 1 run(s)
        Tests: 4 passed, 1 failed, 0 skipped
        
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
        Running fuzzer for fuzzing::tests::fuzzed_argument, 10 runs:
        [PASS] fuzzing::tests::fuzzed_argument
        Running fuzzer for fuzzing::tests::fuzzed_both_arguments, 10 runs:
        [PASS] fuzzing::tests::fuzzed_both_arguments
        [PASS] fuzzing::tests::passing
        Running fuzzer for fuzzing::tests::failing_fuzz, 10 runs:
        [FAIL] fuzzing::tests::failing_fuzz

        Failure data:
            original value: [593979512822486835600413552099926114], converted to a string: [result == a + b]

        Fuzz test failed on argument(s) [[..], [..]] after 1 run(s)
        Tests: 4 passed, 1 failed, 0 skipped
        
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
        Running fuzzer for fuzzing::tests::fuzzed_argument, 256 runs:
        [PASS] fuzzing::tests::fuzzed_argument
        Running fuzzer for fuzzing::tests::fuzzed_both_arguments, 256 runs:
        [PASS] fuzzing::tests::fuzzed_both_arguments
        [PASS] fuzzing::tests::passing
        Running fuzzer for fuzzing::tests::failing_fuzz, 256 runs:
        [FAIL] fuzzing::tests::failing_fuzz

        Failure data:
            original value: [593979512822486835600413552099926114], converted to a string: [result == a + b]

        Fuzz test failed on argument(s) [2747212248768723701292547667432253102957931518300200682643074373162842712217, 2464267667796943162905983180301451257796003176364505486980188124687118920211] after 1 run(s)
        Tests: 4 passed, 1 failed, 0 skipped
        
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
