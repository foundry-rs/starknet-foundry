use super::common::runner::{runner, setup_package};
use crate::{assert_cleaned_output, assert_png_snapshot};
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
fn snap_optimize_inlining_dry_run() {
    let temp = setup_package("simple_package");

    let output = runner(&temp)
        .env("SCARB_UI_VERBOSITY", "quiet")
        .env("SNFORGE_DETERMINISTIC_OUTPUT", "1")
        .arg("optimize-inlining")
        .arg("--exact")
        .arg("simple_package_integrationtest::contract::call_and_invoke")
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

    assert_cleaned_output!(output);

    let graph_bytes = read_optimization_graph(temp.path(), 0, 100, 50);
    assert_png_snapshot!("optimize_inlining_dry_run.png", graph_bytes);
}

#[test]
fn snap_optimize_inlining_updates_manifest() {
    let temp = setup_package("simple_package");

    let initial_scarb_toml = fs::read_to_string(temp.path().join("Scarb.toml"))
        .expect("Failed to read initial Scarb.toml");
    let initial_scarb_toml =
        Document::parse(&initial_scarb_toml).expect("Failed to parse initial Scarb.toml");

    let initial_inlining_strategy = initial_scarb_toml
        .as_table()
        .get("profile")
        .and_then(|item| item.as_table())
        .and_then(|profile| profile.get("release"))
        .and_then(|item| item.as_table())
        .and_then(|release| release.get("cairo"))
        .and_then(|item| item.as_table())
        .and_then(|cairo| cairo.get("inlining-strategy"))
        .and_then(toml_edit::Item::as_integer);
    assert!(
        initial_inlining_strategy.is_none(),
        "inlining-strategy should not be set before optimization"
    );

    let output = runner(&temp)
        .env("SCARB_UI_VERBOSITY", "quiet")
        .env("SNFORGE_DETERMINISTIC_OUTPUT", "1")
        .arg("optimize-inlining")
        .arg("--exact")
        .arg("simple_package_integrationtest::contract::call_and_invoke")
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

    assert_cleaned_output!(output);

    let graph_bytes = read_optimization_graph(temp.path(), 0, 10, 10);
    assert_png_snapshot!("optimize_inlining_updates_manifest.png", graph_bytes);

    let scarb_toml = fs::read_to_string(temp.path().join("Scarb.toml"))
        .expect("Failed to read updated Scarb.toml");
    let scarb_toml = Document::parse(&scarb_toml).expect("Failed to parse updated Scarb.toml");

    let updated_inlining_strategy = scarb_toml
        .as_table()
        .get("profile")
        .and_then(|item| item.as_table())
        .and_then(|profile| profile.get("release"))
        .and_then(|item| item.as_table())
        .and_then(|release| release.get("cairo"))
        .and_then(|item| item.as_table())
        .and_then(|cairo| cairo.get("inlining-strategy"))
        .and_then(toml_edit::Item::as_integer);

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
        .arg("simple_package_integrationtest::contract::call_and_invoke")
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
fn optimize_inlining_fails_with_low_max_program_len() {
    let temp = setup_package("simple_package");

    let output = runner(&temp)
        .arg("optimize-inlining")
        .arg("--exact")
        .arg("simple_package_integrationtest::contract::call_and_invoke")
        .arg("--contracts")
        .arg("HelloStarknet")
        .arg("--max-contract-program-len")
        .arg("1")
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
        [1/1] Testing threshold 0...
        [..]
          ✗ Contract size [..] exceeds limit [..] or felts [..] exceeds limit 1.
        Try optimizing with lower threshold limit.
        [ERROR] Optimization failed: No valid optimization results found
        "},
    );
}

#[test]
fn optimize_inlining_fails_when_no_tests_matched() {
    let temp = setup_package("simple_package");

    let output = runner(&temp)
        .arg("optimize-inlining")
        .arg("--exact")
        .arg("simple_package::tests::nonexistent_test")
        .arg("--contracts")
        .arg("HelloStarknet")
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
        "[ERROR] Optimization failed: No tests were executed. The --exact filter did not match any test cases.",
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
