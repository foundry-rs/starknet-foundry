use crate::coverage_api::run_coverage;
use crate::forge_config::{ExecutionDataToSave, ForgeConfig, TestRunnerConfig};
use crate::fuzzer::RandomFuzzer;
use crate::running::{run_fuzz_test, run_test};
use crate::test_case_summary::TestCaseSummary;
use anyhow::{anyhow, Result};
use build_trace_data::save_trace_data;
use cairo_lang_sierra::program::{ConcreteTypeLongId, Function, TypeDeclaration};
use camino::Utf8PathBuf;
use cheatnet::runtime_extensions::forge_config_extension::config::RawFuzzerConfig;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use package_tests::with_config_resolved::TestCaseWithResolvedConfig;
use profiler_api::run_profiler;
use shared::print::print_as_warning;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use test_case_summary::{AnyTestCaseSummary, Fuzzing};
use tokio::sync::mpsc::{channel, Sender};
use tokio::task::JoinHandle;
use universal_sierra_compiler_api::AssembledProgramWithDebugInfo;

pub mod build_trace_data;
pub mod coverage_api;
pub mod expected_result;
pub mod forge_config;
pub mod package_tests;
pub mod profiler_api;
pub mod test_case_summary;
pub mod test_target_summary;

mod backtrace;
mod fuzzer;
mod gas;
pub mod printing;
pub mod running;

pub const CACHE_DIR: &str = ".snfoundry_cache";

const BUILTINS: [&str; 11] = [
    "Pedersen",
    "RangeCheck",
    "Bitwise",
    "EcOp",
    "Poseidon",
    "SegmentArena",
    "GasBuiltin",
    "System",
    "RangeCheck96",
    "AddMod",
    "MulMod",
];

pub trait TestCaseFilter {
    fn should_be_run(&self, test_case: &TestCaseWithResolvedConfig) -> bool;
}

pub fn maybe_save_trace_and_profile(
    result: &AnyTestCaseSummary,
    execution_data_to_save: &ExecutionDataToSave,
) -> Result<Option<PathBuf>> {
    if let AnyTestCaseSummary::Single(TestCaseSummary::Passed {
        name, trace_data, ..
    }) = result
    {
        if execution_data_to_save.is_vm_trace_needed() {
            let name = sanitize_filename::sanitize(name.replace("::", "_"));
            let trace_path = save_trace_data(&name, trace_data)?;
            if execution_data_to_save.profile {
                run_profiler(&name, &trace_path, &execution_data_to_save.additional_args)?;
            }
            return Ok(Some(trace_path));
        }
    }
    Ok(None)
}

pub fn maybe_generate_coverage(
    execution_data_to_save: &ExecutionDataToSave,
    saved_trace_data_paths: &[PathBuf],
) -> Result<()> {
    if execution_data_to_save.coverage {
        if saved_trace_data_paths.is_empty() {
            print_as_warning(&anyhow!("No trace data to generate coverage from"));
        } else {
            run_coverage(
                saved_trace_data_paths,
                &execution_data_to_save.additional_args,
            )?;
        }
    }
    Ok(())
}

#[must_use]
pub fn run_for_test_case(
    args: Vec<ConcreteTypeLongId>,
    case: Arc<TestCaseWithResolvedConfig>,
    casm_program: Arc<AssembledProgramWithDebugInfo>,
    forge_config: Arc<ForgeConfig>,
    versioned_program_path: Arc<Utf8PathBuf>,
    send: Sender<()>,
) -> JoinHandle<Result<AnyTestCaseSummary>> {
    if args.is_empty() {
        tokio::task::spawn(async move {
            let res = run_test(
                case,
                casm_program,
                forge_config.test_runner_config.clone(),
                versioned_program_path,
                send,
            )
            .await?;
            Ok(AnyTestCaseSummary::Single(res))
        })
    } else {
        tokio::task::spawn(async move {
            let res = run_with_fuzzing(
                args,
                case,
                casm_program,
                forge_config.test_runner_config.clone(),
                versioned_program_path,
                send,
            )
            .await??;
            Ok(AnyTestCaseSummary::Fuzzing(res))
        })
    }
}

fn argument_type_name(arg: &ConcreteTypeLongId) -> &str {
    let name = arg.generic_id.0.as_str();

    if name == "Struct" {
        if let cairo_lang_sierra::program::GenericArg::UserType(
            cairo_lang_sierra::ids::UserTypeId { ref debug_name, .. },
        ) = arg.generic_args[0]
        {
            debug_name.as_ref().unwrap().as_str()
        } else {
            name
        }
    } else {
        name
    }
}

fn run_with_fuzzing(
    args: Vec<ConcreteTypeLongId>,
    case: Arc<TestCaseWithResolvedConfig>,
    casm_program: Arc<AssembledProgramWithDebugInfo>,
    test_runner_config: Arc<TestRunnerConfig>,
    versioned_program_path: Arc<Utf8PathBuf>,
    send: Sender<()>,
) -> JoinHandle<Result<TestCaseSummary<Fuzzing>>> {
    tokio::task::spawn(async move {
        if send.is_closed() {
            return Ok(TestCaseSummary::Skipped {});
        }

        let (fuzzing_send, mut fuzzing_rec) = channel(1);
        let arg_types = args.iter().map(argument_type_name).collect::<Vec<_>>();

        let (fuzzer_runs, fuzzer_seed) = match case.config.fuzzer_config {
            Some(RawFuzzerConfig { runs, seed }) => (
                runs.unwrap_or(test_runner_config.fuzzer_runs),
                seed.unwrap_or(test_runner_config.fuzzer_seed),
            ),
            _ => (
                test_runner_config.fuzzer_runs,
                test_runner_config.fuzzer_seed,
            ),
        };
        let mut fuzzer = RandomFuzzer::create(fuzzer_seed, fuzzer_runs, &arg_types)?;

        let mut tasks = FuturesUnordered::new();

        for _ in 1..=fuzzer_runs.get() {
            let args = fuzzer.next_args();

            tasks.push(run_fuzz_test(
                args,
                case.clone(),
                casm_program.clone(),
                test_runner_config.clone(),
                versioned_program_path.clone(),
                send.clone(),
                fuzzing_send.clone(),
            ));
        }

        let mut results = vec![];
        while let Some(task) = tasks.next().await {
            let result = task?;

            results.push(result.clone());

            if let TestCaseSummary::Failed { .. } = result {
                fuzzing_rec.close();
                break;
            }
        }

        let runs = u32::try_from(
            results
                .iter()
                .filter(|item| {
                    matches!(
                        item,
                        TestCaseSummary::Passed { .. } | TestCaseSummary::Failed { .. }
                    )
                })
                .count(),
        )?;

        let fuzzing_run_summary: TestCaseSummary<Fuzzing> = TestCaseSummary::from(results);

        if let TestCaseSummary::Passed { .. } = fuzzing_run_summary {
            // Because we execute tests parallel, it's possible to
            // get Passed after Skipped. To treat fuzzing a test as Passed
            // we have to ensure that all fuzzing subtests Passed
            if runs != fuzzer_runs.get() {
                return Ok(TestCaseSummary::Skipped {});
            };
        };

        Ok(fuzzing_run_summary)
    })
}

#[allow(clippy::implicit_hasher)]
#[must_use]
pub fn function_args(
    function: &Function,
    type_declarations: &HashMap<u64, &TypeDeclaration>,
) -> Vec<ConcreteTypeLongId> {
    function
        .signature
        .param_types
        .iter()
        .filter_map(|pt| match type_declarations.get(&pt.id) {
            Some(ty_declaration)
                if !BUILTINS.contains(&ty_declaration.long_id.generic_id.0.as_str()) =>
            {
                Some(ty_declaration.long_id.clone())
            }
            _ => None,
        })
        .collect()
}
