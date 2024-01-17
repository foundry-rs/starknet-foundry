use std::collections::HashMap;
use std::default::Default;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::compiled_runnable::ValidatedForkConfig;
use crate::gas::calculate_used_gas;
use crate::sierra_casm_runner::SierraCasmRunner;
use crate::test_case_summary::{Single, TestCaseSummary};
use crate::{RunnerConfig, RunnerParams, TestCaseRunnable, CACHE_DIR};
use anyhow::{bail, ensure, Result};
use blockifier::execution::common_hints::ExecutionMode;
use blockifier::execution::entry_point::{EntryPointExecutionContext, ExecutionResources};
use blockifier::execution::execution_utils::ReadOnlySegments;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::cached_state::CachedState;
use blockifier::state::state_api::State;
use cairo_felt::Felt252;
use cairo_lang_casm::hints::Hint;
use cairo_lang_casm::instructions::Instruction;
use cairo_lang_runner::casm_run::hint_to_hint_params;
use cairo_lang_runner::{Arg, RunResult, RunnerError};
use cairo_vm::serde::deserialize_program::HintParams;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::vm_core::VirtualMachine;
use camino::Utf8Path;
use cheatnet::constants as cheatnet_constants;
use cheatnet::constants::build_test_entry_point;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::CallToBlockifierExtension;
use cheatnet::runtime_extensions::cheatable_starknet_runtime_extension::CheatableStarknetRuntimeExtension;
use cheatnet::runtime_extensions::forge_runtime_extension::{
    get_all_execution_resources, ForgeExtension, ForgeRuntime,
};
use cheatnet::runtime_extensions::io_runtime_extension::IORuntimeExtension;
use cheatnet::state::{BlockInfoReader, CheatnetState, ExtendedStateReader};
use itertools::chain;
use runtime::starknet::context;
use runtime::starknet::context::BlockInfo;
use runtime::{ExtendedRuntime, StarknetRuntime};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

/// Builds `hints_dict` required in `cairo_vm::types::program::Program` from instructions.
fn build_hints_dict<'b>(
    instructions: impl Iterator<Item = &'b Instruction>,
) -> (HashMap<usize, Vec<HintParams>>, HashMap<String, Hint>) {
    let mut hints_dict: HashMap<usize, Vec<HintParams>> = HashMap::new();
    let mut string_to_hint: HashMap<String, Hint> = HashMap::new();

    let mut hint_offset = 0;

    for instruction in instructions {
        if !instruction.hints.is_empty() {
            // Register hint with string for the hint processor.
            for hint in &instruction.hints {
                string_to_hint.insert(format!("{hint:?}"), hint.clone());
            }
            // Add hint, associated with the instruction offset.
            hints_dict.insert(
                hint_offset,
                instruction.hints.iter().map(hint_to_hint_params).collect(),
            );
        }
        hint_offset += instruction.body.op_size();
    }
    (hints_dict, string_to_hint)
}

pub fn run_test(
    case: Arc<TestCaseRunnable>,
    runner: Arc<SierraCasmRunner>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    send: Sender<()>,
) -> JoinHandle<Result<TestCaseSummary<Single>>> {
    tokio::task::spawn_blocking(move || {
        // Due to the inability of spawn_blocking to be abruptly cancelled,
        // a channel is used to receive information indicating
        // that the execution of the task is no longer necessary.
        if send.is_closed() {
            return Ok(TestCaseSummary::Skipped {});
        }
        let run_result = run_test_case(vec![], &case, &runner, &runner_config, &runner_params);

        // TODO: code below is added to fix snforge tests
        // remove it after improve exit-first tests
        // issue #1043
        if send.is_closed() {
            return Ok(TestCaseSummary::Skipped {});
        }

        extract_test_case_summary(run_result, &case, vec![])
    })
}

pub(crate) fn run_fuzz_test(
    args: Vec<Felt252>,
    case: Arc<TestCaseRunnable>,
    runner: Arc<SierraCasmRunner>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    send: Sender<()>,
    fuzzing_send: Sender<()>,
) -> JoinHandle<Result<TestCaseSummary<Single>>> {
    tokio::task::spawn_blocking(move || {
        // Due to the inability of spawn_blocking to be abruptly cancelled,
        // a channel is used to receive information indicating
        // that the execution of the task is no longer necessary.
        if send.is_closed() | fuzzing_send.is_closed() {
            return Ok(TestCaseSummary::Skipped {});
        }

        let run_result =
            run_test_case(args.clone(), &case, &runner, &runner_config, &runner_params);

        // TODO: code below is added to fix snforge tests
        // remove it after improve exit-first tests
        // issue #1043
        if send.is_closed() {
            return Ok(TestCaseSummary::Skipped {});
        }

        extract_test_case_summary(run_result, &case, args)
    })
}

fn build_context(block_info: BlockInfo) -> EntryPointExecutionContext {
    let block_context = context::build_block_context(block_info);
    let account_context = context::build_transaction_context();

    EntryPointExecutionContext::new(
        &block_context,
        &account_context,
        ExecutionMode::Execute,
        false,
    )
    .unwrap()
}

fn build_syscall_handler<'a>(
    blockifier_state: &'a mut dyn State,
    string_to_hint: &'a HashMap<String, Hint>,
    execution_resources: &'a mut ExecutionResources,
    context: &'a mut EntryPointExecutionContext,
) -> SyscallHintProcessor<'a> {
    let entry_point = build_test_entry_point();

    SyscallHintProcessor::new(
        blockifier_state,
        execution_resources,
        context,
        // This segment is created by SierraCasmRunner
        Relocatable {
            segment_index: 10,
            offset: 0,
        },
        entry_point,
        string_to_hint,
        ReadOnlySegments::default(),
    )
}

pub struct RunResultWithInfo {
    pub(crate) run_result: Result<RunResult, RunnerError>,
    pub(crate) gas_used: u128,
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
pub fn run_test_case(
    args: Vec<Felt252>,
    case: &TestCaseRunnable,
    runner: &SierraCasmRunner,
    runner_config: &Arc<RunnerConfig>,
    runner_params: &Arc<RunnerParams>,
) -> Result<RunResultWithInfo> {
    ensure!(
        case.available_gas.is_none(),
        "\n    Attribute `available_gas` is not supported\n"
    );
    let available_gas = Some(usize::MAX);

    let func = runner.find_function(case.name.as_str()).unwrap();
    let initial_gas = runner
        .get_initial_available_gas(func, available_gas)
        .unwrap();
    let runner_args: Vec<Arg> = args.into_iter().map(Arg::Value).collect();

    let (entry_code, builtins) = runner
        .create_entry_code(func, &runner_args, initial_gas)
        .unwrap();
    let footer = SierraCasmRunner::create_code_footer();
    let instructions = chain!(
        entry_code.iter(),
        runner.get_casm_program().instructions.iter(),
        footer.iter()
    );
    let (hints_dict, string_to_hint) = build_hints_dict(instructions.clone());

    let mut state_reader = ExtendedStateReader {
        dict_state_reader: cheatnet_constants::build_testing_state(),
        fork_state_reader: get_fork_state_reader(&runner_config.workspace_root, &case.fork_config),
    };
    let block_info = state_reader.get_block_info()?;

    let mut context = build_context(block_info);
    let mut execution_resources = ExecutionResources::default();
    let mut blockifier_state = CachedState::from(state_reader);
    let syscall_handler = build_syscall_handler(
        &mut blockifier_state,
        &string_to_hint,
        &mut execution_resources,
        &mut context,
    );

    let mut cheatnet_state = CheatnetState {
        block_info,
        ..Default::default()
    };

    let cheatable_runtime = ExtendedRuntime {
        extension: CheatableStarknetRuntimeExtension {
            cheatnet_state: &mut cheatnet_state,
        },
        extended_runtime: StarknetRuntime {
            hint_handler: syscall_handler,
        },
    };

    let io_runtime = ExtendedRuntime {
        extension: IORuntimeExtension {
            lifetime: &PhantomData,
        },
        extended_runtime: cheatable_runtime,
    };

    let call_to_blockifier_runtime = ExtendedRuntime {
        extension: CallToBlockifierExtension {
            lifetime: &PhantomData,
        },
        extended_runtime: io_runtime,
    };

    let forge_extension = ForgeExtension {
        environment_variables: &runner_params.environment_variables,
        contracts: &runner_params.contracts,
    };

    let mut forge_runtime = ExtendedRuntime {
        extension: forge_extension,
        extended_runtime: call_to_blockifier_runtime,
    };

    let mut vm = VirtualMachine::new(true);
    let run_result = runner.run_function_with_vm(
        func,
        &mut vm,
        &mut forge_runtime,
        hints_dict,
        instructions,
        builtins,
    );

    let block_context = get_context(&forge_runtime).block_context.clone();
    let execution_resources = get_all_execution_resources(forge_runtime);

    let gas = calculate_used_gas(&block_context, &mut blockifier_state, &execution_resources);

    Ok(RunResultWithInfo {
        run_result,
        gas_used: gas,
    })
}

fn extract_test_case_summary(
    run_result: Result<RunResultWithInfo>,
    case: &TestCaseRunnable,
    args: Vec<Felt252>,
) -> Result<TestCaseSummary<Single>> {
    match run_result {
        Ok(result_with_info) => {
            match result_with_info.run_result {
                Ok(run_result) => Ok(TestCaseSummary::from_run_result_and_info(
                    run_result,
                    case,
                    args,
                    result_with_info.gas_used,
                )),
                // CairoRunError comes from VirtualMachineError which may come from HintException that originates in TestExecutionSyscallHandler
                Err(RunnerError::CairoRunError(error)) => Ok(TestCaseSummary::Failed {
                    name: case.name.clone(),
                    msg: Some(format!(
                        "\n    {}\n",
                        error.to_string().replace(" Custom Hint Error: ", "\n    ")
                    )),
                    arguments: args,
                    test_statistics: (),
                }),
                Err(err) => bail!(err),
            }
        }
        // `ForkStateReader.get_block_info`, `get_fork_state_reader` may return an error
        // unsupported `available_gas` attribute may be specified
        Err(error) => Ok(TestCaseSummary::Failed {
            name: case.name.clone(),
            msg: Some(error.to_string()),
            arguments: args,
            test_statistics: (),
        }),
    }
}

fn get_fork_state_reader(
    workspace_root: &Utf8Path,
    fork_config: &Option<ValidatedForkConfig>,
) -> Option<ForkStateReader> {
    fork_config
        .as_ref()
        .map(|ValidatedForkConfig { url, block_number }| {
            ForkStateReader::new(
                url.clone(),
                *block_number,
                workspace_root.join(CACHE_DIR).as_ref(),
            )
        })
}

fn get_context<'a>(runtime: &'a ForgeRuntime) -> &'a EntryPointExecutionContext {
    runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .hint_handler
        .context
}
