use super::common::runner::{runner, setup_package};
use indoc::indoc;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};

#[test]
#[cfg_attr(feature = "cairo-native", ignore = "Not supported with cairo-native")]
fn optimize_inlining_dry_run() {
    let temp = setup_package("optimize_inlining");

    let output = runner(&temp)
        .arg("optimize-inlining")
        .arg("--exact")
        .arg("optimize_inlining::tests::test_increase_balance")
        .arg("--dry-run")
        .arg("--min-threshold")
        .arg("0")
        .arg("--max-threshold")
        .arg("500")
        .arg("--step")
        .arg("10")
        .assert()
        .success()
        .stdout_eq("");

    assert_stdout_contains(
        output,
        indoc! {r"
        Starting inlining strategy optimization...
        Search range: 0 to 20, step: 10, max contract size: [..]
        [..]Testing threshold 0...
        [..]Testing threshold 10...
        [..]Testing threshold 20...
        Optimization Results:
        [..]Threshold[..]
        Optimal threshold: [..]
        Dry run - Scarb.toml not modified
        "},
    );
}

#[test]
#[cfg_attr(feature = "cairo-native", ignore = "Not supported with cairo-native")]
fn optimize_inlining_updates_manifest() {
    let temp = setup_package("optimize_inlining");

    let output = runner(&temp)
        .arg("optimize-inlining")
        .arg("--exact")
        .arg("optimize_inlining::tests::test_increase_balance")
        .arg("--min-threshold")
        .arg("0")
        .arg("--max-threshold")
        .arg("10")
        .arg("--step")
        .arg("10")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        Starting inlining strategy optimization...
        [..]Testing threshold 0...
        [..]Testing threshold 10...
        Optimization Results:
        [..]Threshold[..]
        Optimal threshold: [..]
        Updated Scarb.toml with inlining-strategy = [..]
        "},
    );
}

#[test]
#[cfg_attr(feature = "cairo-native", ignore = "Not supported with cairo-native")]
fn optimize_inlining_fails_without_contracts() {
    let temp = setup_package("fuzzing");

    let output = runner(&temp)
        .arg("optimize-inlining")
        .arg("--exact")
        .arg("fuzzing::tests::test_fuzz")
        .arg("--dry-run")
        .assert()
        .failure();

    assert_stderr_contains(
        output,
        "Optimization failed: No starknet_artifacts.json found. Only projects with contracts can be optimized.",
    );
}

#[test]
#[cfg_attr(feature = "cairo-native", ignore = "Not supported with cairo-native")]
fn optimize_inlining_requires_single_exact_test_case() {
    let temp = setup_package("optimize_inlining");

    let output = runner(&temp)
        .arg("optimize-inlining")
        .arg("--dry-run")
        .assert()
        .failure();

    assert_stderr_contains(
        output,
        "error: optimize-inlining requires --exact and one exact test case name",
    );
}
