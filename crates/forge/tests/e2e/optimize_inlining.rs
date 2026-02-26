use super::common::runner::{runner, setup_package};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;
use std::fs;

fn read_optimization_graph(dir: &std::path::Path, min: u32, max: u32, step: u32) -> Vec<u8> {
    let workspace_name = dir.file_name().unwrap().to_string_lossy();
    let filename = format!("{workspace_name}_optimization_results_l_{min}_h_{max}_s_{step}.png");
    fs::read(dir.join("target").join(&filename))
        .unwrap_or_else(|e| panic!("Failed to read {filename}: {e}"))
}

#[test]
fn optimize_inlining_dry_run() {
    let temp = setup_package("optimize_inlining");

    let output = runner(&temp)
        .arg("optimize-inlining")
        .arg("--exact")
        .arg("optimize_inlining::tests::test_increase_balance")
        .arg("--min-threshold")
        .arg("0")
        .arg("--max-threshold")
        .arg("100")
        .arg("--step")
        .arg("50")
        .assert()
        .success();

    assert_stdout_contains(
        output,
        indoc! {r"
        Starting inlining strategy optimization...
        Search range: 0 to 100, step: 50, max contract size: [..] bytes, max felts: [..]
        Copying project to temporary directory...
        Working in: [..]
        Running boundary tests...

        Testing min threshold 0...
        [..]

        Testing max threshold 100...
        [..]

        Optimization Results:
        [..]Threshold[..]
        Lowest runtime gas cost: [..]
        Scarb.toml not modified. Use --gas or --size to apply a threshold.
        "},
    );

    let graph_bytes = read_optimization_graph(temp.path(), 0, 100, 50);
    insta::assert_binary_snapshot!("optimize_inlining_dry_run.png", graph_bytes);
}

#[test]
fn optimize_inlining_updates_manifest() {
    let temp = setup_package("optimize_inlining");

    let output = runner(&temp)
        .arg("optimize-inlining")
        .arg("--exact")
        .arg("optimize_inlining::tests::test_increase_balance")
        .arg("--gas")
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
        Search range: 0 to 10, step: 10, max contract size: [..] bytes, max felts: [..]
        Copying project to temporary directory...
        Working in: [..]
        Running boundary tests...

        Testing min threshold 0...
        [..]

        Testing max threshold 10...
        [..]

        Optimization Results:
        [..]Threshold[..]
        Lowest runtime gas cost: [..]
        Updated Scarb.toml with inlining-strategy = [..]
        "},
    );

    let graph_bytes = read_optimization_graph(temp.path(), 0, 10, 10);
    insta::assert_binary_snapshot!("optimize_inlining_updates_manifest.png", graph_bytes);
}

#[test]
fn optimize_inlining_fails_without_contracts() {
    let temp = setup_package("fuzzing");

    let output = runner(&temp)
        .arg("optimize-inlining")
        .arg("--exact")
        .arg("fuzzing::tests::test_fuzz")
        .assert()
        .failure();

    assert_stdout_contains(
        output,
        "[ERROR] Optimization failed: No starknet_artifacts.json found. Only projects with contracts can be optimized.",
    );
}

#[test]
fn optimize_inlining_requires_single_exact_test_case() {
    let temp = setup_package("optimize_inlining");

    let output = runner(&temp).arg("optimize-inlining").assert().failure();

    assert_stdout_contains(
        output,
        "[ERROR] optimize-inlining requires --exact and one exact test case name",
    );
}
