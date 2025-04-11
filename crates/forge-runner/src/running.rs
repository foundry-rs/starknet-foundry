use crate::backtrace::add_backtrace_footer;
use crate::forge_config::{RuntimeConfig, TestRunnerConfig};
use crate::gas::calculate_used_gas;
use crate::package_tests::with_config_resolved::{ResolvedForkConfig, TestCaseWithResolvedConfig};
use crate::test_case_summary::{Single, TestCaseSummary};
use anyhow::{Result, bail};
use blockifier::execution::call_info::CallInfo;
use blockifier::execution::contract_class::{EntryPointV1, TrackedResource};
use blockifier::execution::entry_point::{EntryPointExecutionContext, ExecutableCallEntryPoint};
use blockifier::execution::errors::{EntryPointExecutionError, PreExecutionError};
use blockifier::execution::execution_utils::ReadOnlySegments;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::cached_state::CachedState;
use blockifier::state::state_api::State;
use cairo_lang_casm::hints::Hint;
use cairo_lang_runner::{Arg, RunResultValue};
use cairo_vm::Felt252;
use cairo_vm::types::builtin_name::BuiltinName;
use cairo_vm::types::layout_name::LayoutName;
use cairo_vm::types::program::Program;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::cairo_runner::CairoRunner;
use camino::{Utf8Path, Utf8PathBuf};
use cheatnet::constants as cheatnet_constants;
use cheatnet::constants::build_test_entry_point;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::CallToBlockifierExtension;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::UsedResources;
use cheatnet::runtime_extensions::cheatable_starknet_runtime_extension::CheatableStarknetRuntimeExtension;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::runtime_extensions::forge_runtime_extension::{
    ForgeExtension, ForgeRuntime, add_resources_to_top_call, get_all_used_resources,
    update_top_call_l1_resources, update_top_call_resources, update_top_call_vm_trace,
};
use cheatnet::state::{
    BlockInfoReader, CallTrace, CheatnetState, EncounteredError, ExtendedStateReader,
};
use hints::hints_by_representation;
use rand::prelude::StdRng;
use runtime::starknet::context::{build_context, set_max_steps};
use runtime::{ExtendedRuntime, StarknetRuntime};
use starknet_api::deprecated_contract_class::EntryPointOffset;
use starknet_api::execution_resources::GasVector;
use starknet_types_core::felt::Felt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::default::Default;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use universal_sierra_compiler_api::AssembledProgramWithDebugInfo;

mod casm;
pub mod config_run;
mod copied_code;
mod entry_code;
mod hints;
mod syscall_handler;
pub mod with_config;

use crate::running::copied_code::{
    finalize_execution, prepare_call_arguments, prepare_program_extra_data, run_entry_point,
};
use crate::running::syscall_handler::build_syscall_handler;
pub use hints::hints_to_params;
pub use syscall_handler::has_segment_arena;
pub use syscall_handler::syscall_handler_offset;

#[must_use]
pub fn run_test(
    case: Arc<TestCaseWithResolvedConfig>,
    casm_program: Arc<AssembledProgramWithDebugInfo>,
    test_runner_config: Arc<TestRunnerConfig>,
    versioned_program_path: Arc<Utf8PathBuf>,
    send: Sender<()>,
) -> JoinHandle<TestCaseSummary<Single>> {
    tokio::task::spawn_blocking(move || {
        // Due to the inability of spawn_blocking to be abruptly cancelled,
        // a channel is used to receive information indicating
        // that the execution of the task is no longer necessary.
        if send.is_closed() {
            return TestCaseSummary::Skipped {};
        }
        let run_result = run_test_case(
            &case,
            &casm_program,
            &RuntimeConfig::from(&test_runner_config),
            None,
        );

        // TODO: code below is added to fix snforge tests
        // remove it after improve exit-first tests
        // issue #1043
        if send.is_closed() {
            return TestCaseSummary::Skipped {};
        }

        extract_test_case_summary(
            run_result,
            &case,
            &test_runner_config.contracts_data,
            &versioned_program_path,
        )
    })
}

pub(crate) fn run_fuzz_test(
    case: Arc<TestCaseWithResolvedConfig>,
    casm_program: Arc<AssembledProgramWithDebugInfo>,
    test_runner_config: Arc<TestRunnerConfig>,
    versioned_program_path: Arc<Utf8PathBuf>,
    send: Sender<()>,
    fuzzing_send: Sender<()>,
    rng: Arc<Mutex<StdRng>>,
) -> JoinHandle<TestCaseSummary<Single>> {
    tokio::task::spawn_blocking(move || {
        // Due to the inability of spawn_blocking to be abruptly cancelled,
        // a channel is used to receive information indicating
        // that the execution of the task is no longer necessary.
        if send.is_closed() | fuzzing_send.is_closed() {
            return TestCaseSummary::Skipped {};
        }

        let run_result = run_test_case(
            &case,
            &casm_program,
            &Arc::new(RuntimeConfig::from(&test_runner_config)),
            Some(rng),
        );

        // TODO: code below is added to fix snforge tests
        // remove it after improve exit-first tests
        // issue #1043
        if send.is_closed() {
            return TestCaseSummary::Skipped {};
        }

        extract_test_case_summary(
            run_result,
            &case,
            &test_runner_config.contracts_data,
            &versioned_program_path,
        )
    })
}

pub enum RunStatus {
    Success(Vec<Felt252>),
    Panic(Vec<Felt252>),
}

impl From<RunResultValue> for RunStatus {
    fn from(value: RunResultValue) -> Self {
        match value {
            RunResultValue::Success(value) => Self::Success(value),
            RunResultValue::Panic(value) => Self::Panic(value),
        }
    }
}

pub struct RunCompleted {
    pub(crate) status: RunStatus,
    pub(crate) call_trace: Rc<RefCell<CallTrace>>,
    pub(crate) gas_used: GasVector,
    pub(crate) used_resources: UsedResources,
    pub(crate) encountered_errors: Vec<EncounteredError>,
    pub(crate) fuzzer_args: Vec<String>,
}

pub struct VmExecutionContext<'a> {
    pub runner: CairoRunner,
    pub syscall_handler: SyscallHintProcessor<'a>,
    pub initial_syscall_ptr: Relocatable,
    // Additional data required for execution is appended after the program bytecode.
    pub program_extra_data_length: usize,
}

fn initialize_execution_context<'a>(
    call: ExecutableCallEntryPoint,
    hints: &'a HashMap<String, Hint>,
    program: &Program,
    state: &'a mut dyn State,
    context: &'a mut EntryPointExecutionContext,
) -> std::result::Result<VmExecutionContext<'a>, PreExecutionError> {
    // Instantiate Cairo runner.
    let proof_mode = false;
    let trace_enabled = true;
    let mut runner = CairoRunner::new(program, LayoutName::all_cairo, proof_mode, trace_enabled)?;

    runner.initialize_function_runner_cairo_1(&builtins_from_program(program))?;
    let mut read_only_segments = ReadOnlySegments::default();
    let program_extra_data_length = prepare_program_extra_data(
        &mut runner,
        program.data_len(),
        &mut read_only_segments,
        &context.versioned_constants().os_constants.gas_costs,
    )?;

    // Instantiate syscall handler.
    let initial_syscall_ptr = runner.vm.add_memory_segment();
    let syscall_handler = SyscallHintProcessor::new(
        state,
        context,
        initial_syscall_ptr,
        call,
        hints,
        read_only_segments,
    );

    Ok(VmExecutionContext {
        runner,
        syscall_handler,
        initial_syscall_ptr,
        program_extra_data_length,
    })
}

#[allow(clippy::too_many_lines)]
pub struct RunError {
    pub(crate) error: Box<CairoRunError>,
    pub(crate) call_trace: Rc<RefCell<CallTrace>>,
    pub(crate) encountered_errors: Vec<EncounteredError>,
    pub(crate) fuzzer_args: Vec<String>,
}

pub enum RunResult {
    Completed(Box<RunCompleted>),
    Error(RunError),
}

// Builtins is private in program, so we need this workaround
fn builtins_from_program(program: &Program) -> Vec<BuiltinName> {
    program.iter_builtins().copied().collect::<Vec<_>>()
}

fn build_test_call_and_entry_point(
    case: &TestCaseWithResolvedConfig,
    casm_program: &AssembledProgramWithDebugInfo,
    program: &Program,
) -> (ExecutableCallEntryPoint, EntryPointV1) {
    let sierra_instruction_idx = case.test_details.sierra_entry_point_statement_idx;
    let casm_entry_point_offset = casm_program.debug_info[sierra_instruction_idx].0;

    let call = build_test_entry_point();
    let entry_point = EntryPointV1 {
        selector: call.entry_point_selector,
        offset: EntryPointOffset(casm_entry_point_offset),
        builtins: builtins_from_program(program),
    };
    (call, entry_point)
}

#[expect(clippy::too_many_lines)]
pub fn run_test_case(
    case: &TestCaseWithResolvedConfig,
    casm_program: &AssembledProgramWithDebugInfo,
    runtime_config: &RuntimeConfig,
    fuzzer_rng: Option<Arc<Mutex<StdRng>>>,
) -> Result<RunResult> {
    let program = case.try_into_program(casm_program)?;
    let (call, entry_point) = build_test_call_and_entry_point(case, casm_program, &program);

    let mut state_reader = ExtendedStateReader {
        dict_state_reader: cheatnet_constants::build_testing_state(),
        fork_state_reader: get_fork_state_reader(
            runtime_config.cache_dir,
            case.config.fork_config.as_ref(),
        )?,
    };
    let block_info = state_reader.get_block_info()?;
    let chain_id = state_reader.get_chain_id()?;
    let tracked_resource = TrackedResource::from(runtime_config.tracked_resource);
    let mut context = build_context(&block_info, chain_id, &tracked_resource);

    if let Some(max_n_steps) = runtime_config.max_n_steps {
        set_max_steps(&mut context, max_n_steps);
    }
    let mut cached_state = CachedState::new(state_reader);

    let hints = hints_by_representation(&casm_program.assembled_cairo_program);
    let VmExecutionContext {
        mut runner,
        syscall_handler,
        initial_syscall_ptr,
        program_extra_data_length,
    } = initialize_execution_context(
        call.clone(),
        &hints,
        &program,
        &mut cached_state,
        &mut context,
    )?;

    let mut cheatnet_state = CheatnetState {
        block_info,
        ..Default::default()
    };
    cheatnet_state.trace_data.is_vm_trace_needed = runtime_config.is_vm_trace_needed;

    let cheatable_runtime = ExtendedRuntime {
        extension: CheatableStarknetRuntimeExtension {
            cheatnet_state: &mut cheatnet_state,
        },
        extended_runtime: StarknetRuntime {
            hint_handler: syscall_handler,
            user_args: vec![],
        },
    };

    let call_to_blockifier_runtime = ExtendedRuntime {
        extension: CallToBlockifierExtension {
            lifetime: &PhantomData,
        },
        extended_runtime: cheatable_runtime,
    };
    let forge_extension = ForgeExtension {
        environment_variables: runtime_config.environment_variables,
        contracts_data: runtime_config.contracts_data,
        fuzzer_rng,
    };

    let mut forge_runtime = ExtendedRuntime {
        extension: forge_extension,
        extended_runtime: call_to_blockifier_runtime,
    };

    let entry_point_initial_budget = forge_runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .hint_handler
        .base
        .context
        .gas_costs()
        .base
        .entry_point_initial_budget;
    let args = prepare_call_arguments(
        &forge_runtime
            .extended_runtime
            .extended_runtime
            .extended_runtime
            .hint_handler
            .base
            .call
            .clone()
            .into(),
        &mut runner,
        initial_syscall_ptr,
        &mut forge_runtime
            .extended_runtime
            .extended_runtime
            .extended_runtime
            .hint_handler
            .read_only_segments,
        &entry_point,
        entry_point_initial_budget,
    )?;

    let n_total_args = args.len();

    // Execute.
    let bytecode_length = program.data_len();
    let program_segment_size = bytecode_length + program_extra_data_length;
    let result: Result<CallInfo, CairoRunError> = match run_entry_point(
        &mut runner,
        &mut forge_runtime,
        entry_point,
        args,
        program_segment_size,
    ) {
        Ok(()) => {
            let call_info = finalize_execution(
                &mut runner,
                &mut forge_runtime
                    .extended_runtime
                    .extended_runtime
                    .extended_runtime
                    .hint_handler,
                n_total_args,
                program_extra_data_length,
                tracked_resource,
            )?;

            // TODO this can be done better, we can take gas directly from call info
            let vm_resources_without_inner_calls = runner
                .get_execution_resources()
                .expect("Execution resources missing")
                .filter_unused_builtins();

            add_resources_to_top_call(
                &mut forge_runtime,
                &vm_resources_without_inner_calls,
                &tracked_resource,
            );

            update_top_call_vm_trace(&mut forge_runtime, &mut runner);

            Ok(call_info)
        }
        Err(error) => Err(match error {
            // TODO verify this mapping
            EntryPointExecutionError::CairoRunError(CairoRunError::VmException(err)) => {
                CairoRunError::VirtualMachine(err.inner_exc)
            }
            EntryPointExecutionError::CairoRunError(err) => err,
            err => bail!(err),
        }),
    };

    let encountered_errors = forge_runtime
        .extended_runtime
        .extended_runtime
        .extension
        .cheatnet_state
        .encountered_errors
        .clone();

    let call_trace_ref = get_call_trace_ref(&mut forge_runtime);

    update_top_call_resources(&mut forge_runtime, &tracked_resource);
    update_top_call_l1_resources(&mut forge_runtime);

    let fuzzer_args = forge_runtime
        .extended_runtime
        .extended_runtime
        .extension
        .cheatnet_state
        .fuzzer_args
        .clone();

    let transaction_context = get_context(&forge_runtime).tx_context.clone();
    let used_resources =
        get_all_used_resources(forge_runtime, &transaction_context, tracked_resource);
    let gas = calculate_used_gas(
        &transaction_context,
        &mut cached_state,
        used_resources.clone(),
    )?;

    Ok(match result {
        Ok(result) => RunResult::Completed(Box::new(RunCompleted {
            status: if result.execution.failed {
                RunStatus::Panic(result.execution.retdata.0)
            } else {
                RunStatus::Success(result.execution.retdata.0)
            },
            call_trace: call_trace_ref,
            gas_used: gas,
            used_resources,
            encountered_errors,
            fuzzer_args,
        })),
        Err(error) => RunResult::Error(RunError {
            error: Box::new(error),
            call_trace: call_trace_ref,
            encountered_errors,
            fuzzer_args,
        }),
    })
}

fn extract_test_case_summary(
    run_result: Result<RunResult>,
    case: &TestCaseWithResolvedConfig,
    contracts_data: &ContractsData,
    versioned_program_path: &Utf8Path,
) -> TestCaseSummary<Single> {
    match run_result {
        Ok(run_result) => match run_result {
            RunResult::Completed(run_completed) => TestCaseSummary::from_run_completed(
                *run_completed,
                case,
                contracts_data,
                versioned_program_path,
            ),
            RunResult::Error(run_error) => {
                let mut message = format!(
                    "\n    {}\n",
                    run_error
                        .error
                        .to_string()
                        .replace(" Custom Hint Error: ", "\n    ")
                );
                if let CairoRunError::VirtualMachine(VirtualMachineError::UnfinishedExecution) =
                    *run_error.error
                {
                    message.push_str(
                        "\n    Suggestion: Consider using the flag `--max-n-steps` to increase allowed limit of steps",
                    );
                }
                TestCaseSummary::Failed {
                    name: case.name.clone(),
                    msg: Some(message).map(|msg| {
                        add_backtrace_footer(msg, contracts_data, &run_error.encountered_errors)
                    }),
                    fuzzer_args: run_error.fuzzer_args,
                    test_statistics: (),
                    debugging_trace: cfg!(feature = "debugging").then(|| {
                        debugging::Trace::new(
                            &run_error.call_trace.borrow(),
                            contracts_data,
                            case.name.clone(),
                        )
                    }),
                }
            }
        },
        // `ForkStateReader.get_block_info`, `get_fork_state_reader, `calculate_used_gas` may return an error
        // `available_gas` may be specified with Scarb ~2.4
        Err(error) => TestCaseSummary::Failed {
            name: case.name.clone(),
            msg: Some(error.to_string()),
            fuzzer_args: Vec::default(),
            test_statistics: (),
            debugging_trace: None,
        },
    }
}

fn get_fork_state_reader(
    cache_dir: &Utf8Path,
    fork_config: Option<&ResolvedForkConfig>,
) -> Result<Option<ForkStateReader>> {
    fork_config
        .as_ref()
        .map(|ResolvedForkConfig { url, block_number }| {
            ForkStateReader::new(url.clone(), *block_number, cache_dir)
        })
        .transpose()
}

fn get_context<'a>(runtime: &'a ForgeRuntime) -> &'a EntryPointExecutionContext {
    runtime
        .extended_runtime
        .extended_runtime
        .extended_runtime
        .hint_handler
        .base
        .context
}

fn get_call_trace_ref(runtime: &mut ForgeRuntime) -> Rc<RefCell<CallTrace>> {
    runtime
        .extended_runtime
        .extended_runtime
        .extension
        .cheatnet_state
        .trace_data
        .current_call_stack
        .top()
}
