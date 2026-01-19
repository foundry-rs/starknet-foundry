use super::common::runner::{runner, setup_package};
use indoc::indoc;
use shared::test_utils::output_assert::assert_stdout_contains;
use std::fs;
use toml_edit::Document;

fn read_optimization_graph(dir: &std::path::Path, min: u32, max: u32, step: u32) -> Vec<u8> {
    let workspace_name = dir.file_name().unwrap().to_string_lossy();
    let filename = format!("{workspace_name}_optimization_results_l_{min}_h_{max}_s_{step}.png");
    fs::read(dir.join("target").join(&filename))
        .unwrap_or_else(|e| panic!("Failed to read {filename}: {e}"))
}

#[test]
fn optimize_inlining_dry_run() {
    let temp = setup_package("simple_package");

    let output = runner(&temp)
        .arg("optimize-inlining")
        .arg("--exact")
        .arg("simple_package::tests::test_increase_balance")
        .arg("--contracts")
        .arg("HelloStarknet")
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
        [1/3] Testing threshold 0...
        [..]
        [2/3] Testing threshold 50...
        [..]
        [3/3] Testing threshold 100...
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
    let temp = setup_package("simple_package");

    let initial_scarb_toml = fs::read_to_string(temp.path().join("Scarb.toml"))
        .expect("Failed to read initial Scarb.toml");
    let initial_scarb_toml =
        Document::parse(&initial_scarb_toml).expect("Failed to parse initial Scarb.toml");

    let initial_inlining_strategy = initial_scarb_toml
        .as_table()
        .get("profile")
        .and_then(|item| item.as_table())
        .and_then(|profile| profile.get("dev"))
        .and_then(|item| item.as_table())
        .and_then(|dev| dev.get("cairo"))
        .and_then(|item| item.as_table())
        .and_then(|cairo| cairo.get("inlining-strategy"))
        .and_then(|item| item.as_integer());
    assert!(
        initial_inlining_strategy.is_none(),
        "inlining-strategy should not be set before optimization"
    );

    let output = runner(&temp)
        .arg("optimize-inlining")
        .arg("--exact")
        .arg("simple_package::tests::test_increase_balance")
        .arg("--contracts")
        .arg("HelloStarknet")
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
        [1/2] Testing threshold 0...
        [..]
        [2/2] Testing threshold 10...
        [..]

        Optimization Results:
        [..]Threshold[..]
        Lowest runtime gas cost: [..]
        Updated Scarb.toml with inlining-strategy = 10
        "},
    );

    let graph_bytes = read_optimization_graph(temp.path(), 0, 10, 10);
    insta::assert_binary_snapshot!("optimize_inlining_updates_manifest.png", graph_bytes);

    let scarb_toml = fs::read_to_string(temp.path().join("Scarb.toml"))
        .expect("Failed to read updated Scarb.toml");
    let scarb_toml = Document::parse(&scarb_toml).expect("Failed to parse updated Scarb.toml");

    let updated_inlining_strategy = scarb_toml
        .as_table()
        .get("profile")
        .and_then(|item| item.as_table())
        .and_then(|profile| profile.get("dev"))
        .and_then(|item| item.as_table())
        .and_then(|dev| dev.get("cairo"))
        .and_then(|item| item.as_table())
        .and_then(|cairo| cairo.get("inlining-strategy"))
        .and_then(|item| item.as_integer());

    assert_eq!(
        updated_inlining_strategy,
        Some(10),
        "inlining-strategy should be set to 10 after optimization"
    );
}

#[test]
fn optimize_inlining_fails_without_contracts() {
    let temp = setup_package("fuzzing");

    let output = runner(&temp)
        .arg("optimize-inlining")
        .arg("--exact")
        .arg("fuzzing::tests::test_fuzz")
        .arg("--contracts")
        .arg("SomeContract")
        .assert()
        .failure();

    assert_stdout_contains(
        output,
        "[ERROR] Optimization failed: No starknet_artifacts.json found. Only projects with contracts can be optimized.",
    );
}

#[test]
fn optimize_inlining_fails_with_nonexistent_contract() {
    let temp = setup_package("simple_package");

    let output = runner(&temp)
        .arg("optimize-inlining")
        .arg("--exact")
        .arg("simple_package::tests::test_increase_balance")
        .arg("--contracts")
        .arg("NonExistentContract")
        .arg("--min-threshold")
        .arg("0")
        .arg("--max-threshold")
        .arg("0")
        .arg("--step")
        .arg("1")
        .assert()
        .failure();

    assert_stdout_contains(
        output,
        indoc! {r"
        [ERROR] Optimization failed: The following contracts were not found in starknet artifacts: NonExistentContract. Available contracts: [..]
        "},
    );
}

#[test]
fn optimize_inlining_requires_single_exact_test_case() {
    let temp = setup_package("simple_package");

    let output = runner(&temp)
        .arg("optimize-inlining")
        .arg("--contracts")
        .arg("HelloStarknet")
        .assert()
        .failure();

    assert_stdout_contains(
        output,
        "[ERROR] optimize-inlining requires using the `--exact` flag",
    );
}
