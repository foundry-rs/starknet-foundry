use std::cell::RefCell;
use std::collections::HashMap;
use std::default::Default;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

use crate::compiled_runnable::ValidatedForkConfig;
use crate::gas::calculate_used_gas;
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
use cairo_lang_runner::casm_run::{build_cairo_runner, run_function_with_runner};
use cairo_lang_runner::{
    build_hints_dict, initialize_vm, Arg, RunResult, RunnerError, SierraCasmRunner,
};
use cairo_lang_sierra::extensions::segment_arena::SegmentArenaType;
use cairo_lang_sierra::extensions::NoGenericArgsGenericType;
use cairo_lang_sierra::ids::GenericTypeId;
use cairo_lang_sierra_to_casm::compiler::CairoProgram;
use cairo_vm::types::relocatable::{MaybeRelocatable, Relocatable};
use cairo_vm::vm::vm_core::VirtualMachine;
use camino::Utf8Path;
use cheatnet::constants as cheatnet_constants;
use cheatnet::constants::build_test_entry_point;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::CallToBlockifierExtension;
use cheatnet::runtime_extensions::cheatable_starknet_runtime_extension::CheatableStarknetRuntimeExtension;
use cheatnet::runtime_extensions::forge_runtime_extension::{
    update_test_execution_resources_and_get_them, ForgeExtension, ForgeRuntime,
};
use cheatnet::runtime_extensions::io_runtime_extension::IORuntimeExtension;
use cheatnet::state::{BlockInfoReader, CallTrace, CheatnetState, ExtendedStateReader};
use itertools::chain;
use runtime::starknet::context;
use runtime::starknet::context::BlockInfo;
use runtime::{ExtendedRuntime, StarknetRuntime};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

pub fn run_test(
    case: Arc<TestCaseRunnable>,
    casm_program: Arc<CairoProgram>,
    test_details: Arc<TestDetails>,
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
        let run_result = run_test_case(
            vec![],
            &case,
            &casm_program,
            &test_details,
            &runner_config,
            &runner_params,
        );

        // TODO: code below is added to fix snforge tests
        // remove it after improve exit-first tests
        // issue #1043
        if send.is_closed() {
            return Ok(TestCaseSummary::Skipped {});
        }

        extract_test_case_summary(run_result, &case, vec![])
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_fuzz_test(
    args: Vec<Felt252>,
    case: Arc<TestCaseRunnable>,
    casm_program: Arc<CairoProgram>,
    test_details: Arc<TestDetails>,
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

        let run_result = run_test_case(
            args.clone(),
            &case,
            &casm_program,
            &test_details,
            &runner_config,
            &runner_params,
        );

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

fn get_syscall_segment_index(test_param_types: &[(GenericTypeId, i16)]) -> isize {
    // Segment arena is allocated conditionally, so segment index is automatically moved (+2 segments)
    if test_param_types
        .iter()
        .any(|(ty, _)| ty == &SegmentArenaType::ID)
    {
        12
    } else {
        10
    }
}

fn build_syscall_handler<'a>(
    blockifier_state: &'a mut dyn State,
    string_to_hint: &'a HashMap<String, Hint>,
    execution_resources: &'a mut ExecutionResources,
    context: &'a mut EntryPointExecutionContext,
    syscall_segment_index: isize,
) -> SyscallHintProcessor<'a> {
    let entry_point = build_test_entry_point();

    SyscallHintProcessor::new(
        blockifier_state,
        execution_resources,
        context,
        Relocatable {
            segment_index: syscall_segment_index,
            offset: 0,
        },
        entry_point,
        string_to_hint,
        ReadOnlySegments::default(),
    )
}

pub struct RunResultWithInfo {
    pub(crate) run_result: Result<RunResult, RunnerError>,
    pub(crate) call_trace: Rc<RefCell<CallTrace>>,
    pub(crate) gas_used: u128,
    pub(crate) used_resources: UsedResources,
}

pub struct TestDetails {
    pub entry_point_offset: usize,
    pub parameter_types: Vec<(GenericTypeId, i16)>,
    pub return_types: Vec<(GenericTypeId, i16)>,
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_lines)]
pub fn run_test_case(
    args: Vec<Felt252>,
    case: &TestCaseRunnable,
    casm_program: &CairoProgram,
    test_details: &TestDetails,
    runner_config: &Arc<RunnerConfig>,
    runner_params: &Arc<RunnerParams>,
) -> Result<RunResultWithInfo> {
    ensure!(
        case.available_gas != Some(0),
        "\n\t`available_gas` attribute was incorrectly configured. Make sure you use scarb >= 2.4.4\n"
    );

    let initial_gas = usize::MAX;
    let runner_args: Vec<Arg> = args.into_iter().map(Arg::Value).collect();
    let (entry_code, builtins) = SierraCasmRunner::create_entry_code_from_params(
        &test_details.parameter_types,
        &runner_args,
        initial_gas,
        casm_program.debug_info.sierra_statement_info[test_details.entry_point_offset].code_offset,
    )
    .unwrap();
    let footer = SierraCasmRunner::create_code_footer();
    let instructions = chain!(entry_code.iter(), casm_program.instructions.iter());
    let (hints_dict, string_to_hint) = build_hints_dict(instructions.clone());
    let assembled_program = casm_program.assemble_ex(&entry_code, &footer);

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
        get_syscall_segment_index(&test_details.parameter_types),
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

    let data: Vec<MaybeRelocatable> = assembled_program
        .bytecode
        .iter()
        .map(Felt252::from)
        .map(MaybeRelocatable::from)
        .collect();
    let data_len = data.len();
    let mut runner = build_cairo_runner(data, builtins, hints_dict)?;

    let run_result = match run_function_with_runner(
        &mut vm,
        data_len,
        initialize_vm,
        &mut forge_runtime,
        &mut runner,
    ) {
        Ok(()) => {
            let vm_resources_without_inner_calls = runner
                .get_execution_resources(&vm)
                .unwrap()
                .filter_unused_builtins();
            forge_runtime
                .extended_runtime
                .extended_runtime
                .extended_runtime
                .extended_runtime
                .hint_handler
                .resources
                .vm_resources += &vm_resources_without_inner_calls;

            let cells = runner.relocated_memory;
            let ap = vm.get_relocated_trace().unwrap().last().unwrap().ap;

            let (results_data, gas_counter) =
                SierraCasmRunner::get_results_data(&test_details.return_types, &cells, ap);
            assert_eq!(results_data.len(), 1);

            let (_, values) = results_data[0].clone();
            let value = SierraCasmRunner::handle_main_return_value(
                // Here we assume that all test either panic or do not return any value
                // This is true for all test right now, but in case it changes
                // this logic will need to be updated
                Some(0),
                values,
                &cells,
            );

            Ok(RunResult {
                gas_counter,
                memory: cells,
                value,
                profiling_info: None,
            })
        }
        Err(err) => Err(RunnerError::CairoRunError(err)),
    };

    let block_context = get_context(&forge_runtime).block_context.clone();
    let call_trace_ref = get_call_trace_ref(&mut forge_runtime);
    let execution_resources = update_test_execution_resources_and_get_them(forge_runtime);

    let gas = calculate_used_gas(&block_context, &mut blockifier_state, &execution_resources)?;

    Ok(RunResultWithInfo {
        run_result,
        gas_used: gas,
        used_resources: execution_resources,
        call_trace: call_trace_ref,
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
                    result_with_info.used_resources,
                    &result_with_info.call_trace,
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
        // `ForkStateReader.get_block_info`, `get_fork_state_reader, `calculate_used_gas` may return an error
        // `available_gas` may be specified with Scarb ~2.4
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

fn get_call_trace_ref(runtime: &mut ForgeRuntime) -> Rc<RefCell<CallTrace>> {
    runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .extension
        .cheatnet_state
        .trace_data
        .current_call_stack
        .last()
}
