use crate::optimize_inlining::args::OptimizeInliningArgs;
use crate::optimize_inlining::contract_size::check_contract_sizes;
use crate::optimize_inlining::manifest::ManifestEditor;
use crate::run_tests::workspace::execute_workspace;
use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use forge_runner::test_case_summary::{AnyTestCaseSummary, TestCaseSummary};
use foundry_ui::UI;
use scarb_api::ScarbCommand;
use scarb_api::metadata::Metadata;
use scarb_api::{target_dir_for_workspace, test_targets_by_name};
use std::sync::Arc;
use std::thread::available_parallelism;
use std::{env, fs};
use tokio::runtime::Builder;

#[derive(Debug, Clone, Default)]
pub struct TotalGas {
    pub l1_gas: u64,
    pub l1_data_gas: u64,
    pub l2_gas: u64,
}

impl TotalGas {
    pub fn total(&self) -> u64 {
        self.l2_gas
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub threshold: u32,
    pub total_gas: TotalGas,
    pub max_contract_size: u64,
    pub max_contract_felts: u64,
    pub tests_passed: bool,
    pub error: Option<String>,
}

pub fn run_optimization_iteration(
    threshold: u32,
    args: &OptimizeInliningArgs,
    scarb_metadata: &Metadata,
    ui: &Arc<UI>,
) -> Result<OptimizationResult> {
    let profile = &scarb_metadata.current_profile;

    let manifest_editor = ManifestEditor::new(&scarb_metadata.workspace.manifest_path)?;
    manifest_editor.set_inlining_strategy(threshold, profile)?;

    let build_result = ScarbCommand::new_with_stdio()
        .manifest_path(dbg!(&scarb_metadata.runtime_manifest))
        .arg("--profile")
        .arg(profile)
        .arg("build")
        .arg("-w")
        .arg("--test")
        .run();

    if let Err(e) = build_result {
        return Ok(OptimizationResult {
            threshold,
            total_gas: TotalGas::default(),
            max_contract_size: 0,
            max_contract_felts: 0,
            tests_passed: false,
            error: Some(format!("Build failed: {e}")),
        });
    }

    let artifacts_dir = dbg!(target_dir_for_workspace(scarb_metadata).join(profile));
    // dbg!(fs::read_dir(&artifacts_dir).unwrap().collect::<Vec<_>>());

    let starknet_artifacts_paths =
        find_test_target_starknet_artifacts(&artifacts_dir, scarb_metadata);
    if starknet_artifacts_paths.is_empty() {
        return Err(anyhow::anyhow!(
            "No starknet_artifacts.json found. Only projects with contracts can be optimized."
        ));
    }
    let (sizes_valid, sizes) = check_contract_sizes(
        &starknet_artifacts_paths,
        args.max_contract_size,
        args.max_contract_felts,
    )?;
    let max_contract_size = sizes.iter().map(|s| s.size).max().unwrap_or(0);
    let max_contract_felts = sizes.iter().map(|s| s.felts_count).max().unwrap_or(0);

    if !sizes_valid {
        return Ok(OptimizationResult {
            threshold,
            total_gas: TotalGas::default(),
            max_contract_size,
            max_contract_felts,
            tests_passed: false,
            error: Some(format!(
                "Contract size {} exceeds limit {} or felts {} exceeds limit {}",
                max_contract_size,
                args.max_contract_size,
                max_contract_felts,
                args.max_contract_felts
            )),
        });
    }

    let test_result = run_tests_with_execute_workspace(
        &scarb_metadata.runtime_manifest.clone().parent().unwrap(),
        args,
        ui,
    )?;

    let tests_passed = test_result.success;
    let total_gas = if tests_passed {
        test_result.total_gas
    } else {
        TotalGas::default()
    };

    Ok(OptimizationResult {
        threshold,
        total_gas,
        max_contract_size,
        max_contract_felts,
        tests_passed,
        error: if tests_passed {
            None
        } else {
            Some(
                test_result
                    .error
                    .unwrap_or_else(|| "Some tests failed".to_string()),
            )
        },
    })
}

struct TestRunResult {
    success: bool,
    total_gas: TotalGas,
    error: Option<String>,
}

fn run_tests_with_execute_workspace(
    root: &Utf8Path,
    args: &OptimizeInliningArgs,
    ui: &Arc<UI>,
) -> Result<TestRunResult> {
    let cores = if let Ok(available_cores) = available_parallelism() {
        available_cores.get()
    } else {
        ui.eprintln(&"Failed to get the number of available cores, defaulting to 1");
        1
    };

    let rt = Builder::new_multi_thread()
        .max_blocking_threads(cores)
        .enable_all()
        .build()?;

    env::set_current_dir(root)?;

    let result = rt.block_on(execute_workspace(args.test_args.clone(), ui.clone()));
    dbg!(&result);
    match result {
        Ok(summary) => {
            let mut all_passed = true;
            let mut total_gas = TotalGas::default();
            let mut first_error: Option<String> = None;

            for test_target in &summary.all_tests {
                for test_case in &test_target.test_case_summaries {
                    if test_case.is_failed() {
                        all_passed = false;
                        if first_error.is_none() {
                            first_error = Some(format!(
                                "Test '{}' failed",
                                test_case.name().unwrap_or("unknown")
                            ));
                        }
                    }

                    if test_case.is_passed() {
                        let gas = extract_gas_from_summary(test_case);
                        total_gas.l1_gas += gas.l1_gas;
                        total_gas.l1_data_gas += gas.l1_data_gas;
                        total_gas.l2_gas += gas.l2_gas;
                    }
                }
            }

            Ok(TestRunResult {
                success: all_passed,
                total_gas,
                error: first_error,
            })
        }
        Err(e) => Ok(TestRunResult {
            success: false,
            total_gas: TotalGas::default(),
            error: Some(format!("Test execution failed: {e}")),
        }),
    }
}

fn extract_gas_from_summary(summary: &AnyTestCaseSummary) -> TotalGas {
    match summary {
        AnyTestCaseSummary::Single(TestCaseSummary::Passed { gas_info, .. }) => TotalGas {
            l1_gas: gas_info.gas_used.l1_gas.0,
            l1_data_gas: gas_info.gas_used.l1_data_gas.0,
            l2_gas: gas_info.gas_used.l2_gas.0,
        },
        AnyTestCaseSummary::Fuzzing(TestCaseSummary::Passed { gas_info, .. }) => TotalGas {
            l1_gas: gas_info.l1_gas.mean as u64,
            l1_data_gas: gas_info.l1_data_gas.mean as u64,
            l2_gas: gas_info.l2_gas.mean as u64,
        },
        _ => TotalGas::default(),
    }
}

fn find_test_target_starknet_artifacts(
    artifacts_dir: &camino::Utf8Path,
    scarb_metadata: &Metadata,
) -> Vec<Utf8PathBuf> {
    let mut paths = Vec::new();

    for package in &scarb_metadata.packages {
        let test_targets = test_targets_by_name(package);
        for target_name in test_targets.keys() {
            let artifact_path =
                artifacts_dir.join(format!("{target_name}.test.starknet_artifacts.json"));
            if artifact_path.exists() {
                paths.push(artifact_path);
            }
        }
    }

    paths
}
