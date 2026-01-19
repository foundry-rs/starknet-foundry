use crate::optimize_inlining::args::OptimizeInliningArgs;
use crate::optimize_inlining::contract_size::{
    ContractArtifactType, ContractSizeInfo, check_and_validate_contract_sizes,
};
use crate::run_tests::workspace::execute_workspace;
use anyhow::{Context, Result};
use blockifier::blockifier_versioned_constants::VersionedConstants;
use blockifier::fee::eth_gas_constants::WORD_WIDTH;
use camino::{Utf8Path, Utf8PathBuf};
use forge_runner::test_case_summary::{AnyTestCaseSummary, TestCaseSummary};
use foundry_ui::UI;
use indoc::formatdoc;
use scarb_api::ScarbCommand;
use scarb_api::artifacts::deserialized::artifacts_for_package;
use scarb_api::manifest::ManifestEditor;
use scarb_api::metadata::{Metadata, MetadataOpts, metadata_with_opts};
use scarb_api::{target_dir_for_workspace, test_targets_by_name};
use starknet_api::transaction::fields::GasVectorComputationMode;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::{env, fs};
use tokio::runtime::Builder;

#[derive(Debug, Clone, Default)]
pub struct TotalGas {
    pub l1: f64,
    pub l1_data: f64,
    pub l2: f64,
}

impl TotalGas {
    pub fn l2(&self) -> f64 {
        self.l2
    }
}

#[derive(Debug, Clone)]
pub struct OptimizationIterationResult {
    pub threshold: u32,
    pub total_gas: TotalGas,
    pub max_contract_size: u64,
    pub contract_code_l2_gas: u64,
    pub tests_passed: bool,
    pub error: Option<String>,
}

pub fn compile_default(scarb_metadata: &Metadata, ui: &Arc<UI>) -> Result<()> {
    ui.println(&"Compiling project with default threshold...".to_string());

    let profile = &scarb_metadata.current_profile;

    ScarbCommand::new_with_stdio()
        .manifest_path(&scarb_metadata.runtime_manifest)
        .arg("--profile")
        .arg(profile)
        .arg("build")
        .arg("-w")
        .arg("--test")
        .run()
        .map_err(|e| anyhow::anyhow!("Build failed: {e}"))?;

    let artifacts_dir = target_dir_for_workspace(scarb_metadata).join(profile);
    let saved_dir = target_dir_for_workspace(scarb_metadata).join("inlining_optimizer_artifacts");
    fs::create_dir_all(&saved_dir)?;
    for entry in fs::read_dir(&artifacts_dir).context("Failed to read artifacts directory")? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            let src_path = Utf8PathBuf::try_from(entry.path())
                .context("Non-UTF-8 path in artifacts directory")?;
            let dst_path = saved_dir.join(src_path.file_name().context("Missing file name")?);
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

pub fn run_optimization_iteration(
    threshold: u32,
    args: &OptimizeInliningArgs,
    scarb_metadata: &Metadata,
    cores: usize,
    ui: &Arc<UI>,
) -> Result<OptimizationIterationResult> {
    let profile = &scarb_metadata.current_profile;

    let manifest_editor = ManifestEditor::new(&scarb_metadata.workspace.manifest_path);
    manifest_editor.set_inlining_strategy(threshold, profile)?;

    let build_result = ScarbCommand::new_with_stdio()
        .manifest_path(&scarb_metadata.runtime_manifest)
        .arg("--profile")
        .arg(profile)
        .arg("build")
        .arg("-w")
        .arg("--test")
        .run();

    if let Err(e) = build_result {
        return Ok(OptimizationIterationResult {
            threshold,
            total_gas: TotalGas::default(),
            max_contract_size: 0,
            contract_code_l2_gas: 0,
            tests_passed: false,
            error: Some(format!("Build failed: {e}")),
        });
    }

    let artifacts_dir = target_dir_for_workspace(scarb_metadata).join(profile);

    let starknet_artifacts_paths =
        find_test_target_starknet_artifacts(&artifacts_dir, scarb_metadata)?;
    if starknet_artifacts_paths.is_empty() {
        return Err(anyhow::anyhow!(
            "No starknet_artifacts.json found. Only projects with contracts can be optimized."
        ));
    }

    let saved_dir = target_dir_for_workspace(scarb_metadata).join("inlining_optimizer_artifacts");
    let keep_filenames =
        matching_contract_artifact_filenames(&starknet_artifacts_paths, &args.contracts)?;
    restore_non_contract_artifacts(&artifacts_dir, &saved_dir, &keep_filenames)
        .context("Failed to restore non-contract artifacts from default build")?;

    let (sizes_valid, sizes) = check_and_validate_contract_sizes(
        &starknet_artifacts_paths,
        args.max_contract_size,
        args.max_contract_program_len,
        &args.contracts,
    )?;
    let max_contract_size = sizes.iter().map(|s| s.size).max().unwrap_or(0);
    let contract_code_l2_gas = contract_code_l2_gas(&sizes)?;

    if !sizes_valid {
        return Ok(OptimizationIterationResult {
            threshold,
            total_gas: TotalGas::default(),
            max_contract_size,
            contract_code_l2_gas,
            tests_passed: false,
            error: Some(formatdoc!(
                r"
                Contract size {max_contract_size} exceeds limit {} or felts {} exceeds limit {}.
                Try optimizing with lower threshold limit.
                ",
                args.max_contract_size,
                sizes.iter().map(|s| s.felts_count).max().unwrap_or(0),
                args.max_contract_program_len
            )),
        });
    }

    let test_result = run_tests_with_execute_workspace(
        scarb_metadata.runtime_manifest.clone().parent().unwrap(),
        args,
        cores,
        ui,
    )?;

    let tests_passed = test_result.success;
    let total_gas = if tests_passed {
        test_result.total_gas
    } else {
        TotalGas::default()
    };

    Ok(OptimizationIterationResult {
        threshold,
        total_gas,
        max_contract_size,
        contract_code_l2_gas,
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

/// Estimates the L2 data gas cost of deploying contarct code for all project contracts.
///
/// This estimation is only concerned with the part of L2 data cost, that depends on the compile contract code size.
///
/// We sum Sierra and CASM felt counts per contract, convert felts to bytes
/// (`felt_count * WORD_WIDTH`), then multiply by the `gas_per_code_byte`
/// rate from the latest Starknet versioned constants.
///
/// See <https://docs.starknet.io/learn/protocol/fees#l2-data> for details.
fn contract_code_l2_gas(sizes: &[ContractSizeInfo]) -> Result<u64> {
    let mut felts_by_contract: HashMap<&str, u64> = HashMap::new();

    for size in sizes {
        if matches!(
            size.artifact_type,
            ContractArtifactType::Sierra | ContractArtifactType::Casm
        ) {
            *felts_by_contract
                .entry(size.contract_id.as_str())
                .or_default() += size.felts_count;
        }
    }

    let versioned_constants = VersionedConstants::latest_constants();
    let gas_per_code_byte = versioned_constants
        .get_archival_data_gas_costs(&GasVectorComputationMode::All)
        .gas_per_code_byte;
    let word_width = u64::try_from(WORD_WIDTH).expect("WORD_WIDTH should fit into u64");

    felts_by_contract
        .values()
        .try_fold(0_u64, |total, felt_count| {
            let code_size_bytes = felt_count
                .checked_mul(word_width)
                .context("code size in bytes overflowed while calculating L2 gas")?;
            let code_l2_gas = (gas_per_code_byte * code_size_bytes).to_integer();
            total
                .checked_add(code_l2_gas)
                .context("contract code L2 gas overflowed")
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
    cores: usize,
    ui: &Arc<UI>,
) -> Result<TestRunResult> {
    let rt = Builder::new_multi_thread()
        .max_blocking_threads(cores)
        .enable_all()
        .build()?;

    let original_cwd = env::current_dir()?;
    env::set_current_dir(root)?;

    let scarb_metadata = metadata_with_opts(MetadataOpts {
        profile: args.test_args.scarb_args.profile.specified(),
        ..MetadataOpts::default()
    })?;

    let result = rt.block_on(execute_workspace(
        &args.test_args,
        ui.clone(),
        &scarb_metadata,
    ));

    env::set_current_dir(&original_cwd)?;

    match result {
        Ok(summary) => {
            let mut all_passed = true;
            let mut total_gas = TotalGas::default();
            let mut first_error: Option<String> = None;
            let mut tests_run = 0u64;

            for test_target in &summary.all_tests {
                for test_case in &test_target.test_case_summaries {
                    if test_case.is_failed() {
                        tests_run += 1;
                        all_passed = false;
                        if first_error.is_none() {
                            first_error = Some(format!(
                                "Test '{}' failed",
                                test_case.name().unwrap_or("unknown")
                            ));
                        }
                    }

                    if test_case.is_passed() {
                        tests_run += 1;
                        let gas = extract_gas_from_summary(test_case);
                        total_gas.l1 += gas.l1;
                        total_gas.l1_data += gas.l1_data;
                        total_gas.l2 += gas.l2;
                    }
                }
            }

            if tests_run == 0 {
                return Err(anyhow::anyhow!(
                    "No tests were executed. The --exact filter did not match any test cases."
                ));
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

#[allow(clippy::cast_precision_loss)]
fn extract_gas_from_summary(summary: &AnyTestCaseSummary) -> TotalGas {
    match summary {
        AnyTestCaseSummary::Single(TestCaseSummary::Passed { gas_info, .. }) => TotalGas {
            l1: gas_info.gas_used.l1_gas.0 as f64,
            l1_data: gas_info.gas_used.l1_data_gas.0 as f64,
            l2: gas_info.gas_used.l2_gas.0 as f64,
        },
        AnyTestCaseSummary::Fuzzing(TestCaseSummary::Passed { gas_info, .. }) => TotalGas {
            l1: gas_info.l1_gas.mean,
            l1_data: gas_info.l1_data_gas.mean,
            l2: gas_info.l2_gas.mean,
        },
        _ => TotalGas::default(),
    }
}

fn matching_contract_artifact_filenames(
    starknet_artifacts_paths: &[Utf8PathBuf],
    contracts_filter: &[String],
) -> Result<HashSet<String>> {
    let mut filenames = HashSet::new();
    for starknet_artifacts_path in starknet_artifacts_paths {
        let artifacts = artifacts_for_package(starknet_artifacts_path.as_path())?;
        for contract in &artifacts.contracts {
            let matches = contracts_filter.iter().any(|f| {
                contract.contract_name == *f
                    || (f.contains("::") && contract.module_path.ends_with(f.as_str()))
            });
            if !matches {
                continue;
            }
            if let Some(name) = contract.artifacts.sierra.file_name() {
                filenames.insert(name.to_owned());
            }
            if let Some(casm) = &contract.artifacts.casm
                && let Some(name) = casm.file_name()
            {
                filenames.insert(name.to_owned());
            }
        }
    }
    Ok(filenames)
}

fn restore_non_contract_artifacts(
    artifacts_dir: &Utf8Path,
    saved_dir: &Utf8Path,
    keep_filenames: &HashSet<String>,
) -> Result<()> {
    for entry in fs::read_dir(saved_dir).context("Failed to read saved artifacts directory")? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let filename = entry.file_name();
        let filename_str = filename.to_string_lossy();
        if keep_filenames.contains(filename_str.as_ref()) {
            continue;
        }
        let src =
            Utf8PathBuf::try_from(entry.path()).context("Non-UTF-8 path in saved artifacts")?;
        let dst = artifacts_dir.join(filename_str.as_ref());
        fs::copy(&src, &dst)?;
    }
    Ok(())
}

fn find_test_target_starknet_artifacts(
    artifacts_dir: &camino::Utf8Path,
    scarb_metadata: &Metadata,
) -> Result<Vec<Utf8PathBuf>> {
    let mut paths = Vec::new();

    for package in &scarb_metadata.packages {
        let test_targets = test_targets_by_name(package);
        for target_name in test_targets.keys() {
            let artifact_path =
                artifacts_dir.join(format!("{target_name}.test.starknet_artifacts.json"));
            if artifact_path.exists() && has_non_empty_contracts_field(&artifact_path)? {
                paths.push(artifact_path);
            }
        }
    }

    Ok(paths)
}

fn has_non_empty_contracts_field(artifact_path: &Utf8Path) -> Result<bool> {
    let artifacts = artifacts_for_package(artifact_path)
        .with_context(|| format!("Failed to load starknet artifacts from {artifact_path}"))?;
    Ok(!artifacts.contracts.is_empty())
}
