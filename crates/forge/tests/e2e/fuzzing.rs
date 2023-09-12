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

        Fuzz test failed on argument(s) [2394994055116509985312196570273447499427990317727499410986793707339898834933, 2221030839115559321431066648854854867047555445217530669564583917731160170757] after 1 run(s)
        Tests: 4 passed, 1 failed, 0 skipped
        
        Failures:
            fuzzing::tests::failing_fuzz
        "#});
}
