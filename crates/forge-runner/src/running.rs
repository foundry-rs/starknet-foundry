use crate::backtrace::add_backtrace_footer;
use crate::forge_config::{RuntimeConfig, TestRunnerConfig};
use crate::gas::calculate_used_gas;
use crate::package_tests::with_config_resolved::{ResolvedForkConfig, TestCaseWithResolvedConfig};
use crate::test_case_summary::{Single, TestCaseSummary};
use anyhow::{Error, Result, bail, ensure};
use blockifier::execution::call_info::{CallExecution, CallInfo};
use blockifier::execution::contract_class::{EntryPointV1, TrackedResource};
use blockifier::execution::entry_point::{
    CallEntryPoint, EntryPointExecutionContext, EntryPointExecutionResult, ExecutableCallEntryPoint,
};
use blockifier::execution::entry_point_execution::CallResult;
use blockifier::execution::errors::{
    EntryPointExecutionError, PostExecutionError, PreExecutionError,
};
use blockifier::execution::execution_utils::{
    Args, ReadOnlySegments, SEGMENT_ARENA_BUILTIN_SIZE, read_execution_retdata, write_felt,
    write_maybe_relocatable,
};
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::cached_state::CachedState;
use blockifier::state::state_api::State;
use blockifier::versioned_constants::GasCosts;
use cairo_lang_casm::hints::Hint;
use cairo_lang_runner::{Arg, RunResultValue, SierraCasmRunner};
use cairo_lang_sierra::extensions::NamedType;
use cairo_lang_sierra::extensions::bitwise::BitwiseType;
use cairo_lang_sierra::extensions::circuit::{AddModType, MulModType};
use cairo_lang_sierra::extensions::ec::EcOpType;
use cairo_lang_sierra::extensions::gas::GasBuiltinType;
use cairo_lang_sierra::extensions::pedersen::PedersenType;
use cairo_lang_sierra::extensions::poseidon::PoseidonType;
use cairo_lang_sierra::extensions::range_check::{RangeCheck96Type, RangeCheckType};
use cairo_lang_sierra::extensions::segment_arena::SegmentArenaType;
use cairo_lang_sierra::extensions::starknet::syscalls::SystemType;
use cairo_lang_sierra::ids::GenericTypeId;
use cairo_lang_utils::Upcast;
use cairo_lang_utils::unordered_hash_set::UnorderedHashSet;
use cairo_vm::Felt252;
use cairo_vm::hint_processor::hint_processor_definition::HintProcessor;
use cairo_vm::serde::deserialize_program::ReferenceManager;
use cairo_vm::types::builtin_name::BuiltinName;
use cairo_vm::types::layout_name::LayoutName;
use cairo_vm::types::program::Program;
use cairo_vm::types::relocatable::{MaybeRelocatable, Relocatable};
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use cairo_vm::vm::errors::memory_errors::MemoryError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use cairo_vm::vm::runners::builtin_runner::BuiltinRunner;
use cairo_vm::vm::runners::cairo_runner::{CairoArg, CairoRunner, ExecutionResources};
use cairo_vm::vm::security::verify_secure_runner;
use camino::{Utf8Path, Utf8PathBuf};
use casm::{get_assembled_program, run_assembled_program};
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
use entry_code::create_entry_code;
use hints::{hints_by_representation, hints_to_params};
use num_traits::{ToPrimitive, Zero};
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
use std::os::macos::raw::stat;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use universal_sierra_compiler_api::AssembledProgramWithDebugInfo;

mod casm;
pub mod config_run;
mod entry_code;
mod hints;
mod syscall_handler;
pub mod with_config;

use crate::running::syscall_handler::build_syscall_handler;
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

fn process_builtins(param_types: &[(GenericTypeId, i16)]) -> Vec<BuiltinName> {
    let mut builtins = vec![];
    println!("param_types: {:?}", param_types);

    // let mut builtin_offset = 3;
    // If modifying this, make sure that the order of builtins matches that from
    // `#[implicit_precedence(...)` in generated test code.
    // 
    // Note the .reverse() below
    for (builtin_name, builtin_ty) in [
        (BuiltinName::mul_mod, MulModType::ID),
        (BuiltinName::add_mod, AddModType::ID),
        (BuiltinName::range_check96, RangeCheck96Type::ID),
        (BuiltinName::segment_arena, SegmentArenaType::ID),
        (BuiltinName::poseidon, PoseidonType::ID),
        (BuiltinName::ec_op, EcOpType::ID),
        (BuiltinName::bitwise, BitwiseType::ID),
        (BuiltinName::range_check, RangeCheckType::ID),
        (BuiltinName::pedersen, PedersenType::ID),
    ] {
        if param_types.iter().any(|(ty, _)| ty == &builtin_ty) {
            // self.input_builtin_vars.insert(
            //     builtin_name,
            //     self.ctx.add_var(CellExpression::Deref(cairo_lang_casm::deref!([fp - builtin_offset]))),
            // );
            // self.builtin_ty_to_vm_name.insert(builtin_ty, builtin_name);
            // builtin_offset += 1;
            builtins.push(builtin_name);
        }
    }
    // TODO not sure if we need this
    // if !self.config.testing {
    //     let output_builtin_var =
    //         self.ctx.add_var(CellExpression::Deref(cairo_lang_casm::deref!([fp - builtin_offset])));
    //     self.input_builtin_vars.insert(BuiltinName::output, output_builtin_var);
    //     self.builtins.push(BuiltinName::output);
    // }
    builtins.reverse();
    builtins
}

pub struct VmExecutionContext<'a> {
    pub runner: CairoRunner,
    pub syscall_handler: SyscallHintProcessor<'a>,
    pub initial_syscall_ptr: Relocatable,
    pub entry_point: EntryPointV1,
    // Additional data required for execution is appended after the program bytecode.
    pub program_extra_data_length: usize,
}

fn prepare_program_extra_data(
    runner: &mut CairoRunner,
    // contract_class: &CompiledClassV1,
    bytecode_length: usize,
    read_only_segments: &mut ReadOnlySegments,
    gas_costs: &GasCosts,
) -> std::result::Result<usize, PreExecutionError> {
    // Create the builtin cost segment, the builtin order should be the same as the price builtin
    // array in the os in compiled_class.cairo in load_compiled_class_facts.
    let builtin_price_array = [
        gas_costs.builtins.pedersen,
        gas_costs.builtins.bitwise,
        gas_costs.builtins.ecop,
        gas_costs.builtins.poseidon,
        gas_costs.builtins.add_mod,
        gas_costs.builtins.mul_mod,
    ];

    let data = builtin_price_array
        .iter()
        .map(|&x| MaybeRelocatable::from(Felt::from(x)))
        .collect::<Vec<_>>();
    let builtin_cost_segment_start = read_only_segments.allocate(&mut runner.vm, &data)?;

    // Put a pointer to the builtin cost segment at the end of the program (after the
    // additional `ret` statement).
    let mut ptr = (runner.vm.get_pc() + bytecode_length)?;
    // Push a `ret` opcode.
    write_felt(
        &mut runner.vm,
        &mut ptr,
        Felt::from(0x208b_7fff_7fff_7ffe_u128),
    )?;
    // Push a pointer to the builtin cost segment.
    write_maybe_relocatable(&mut runner.vm, &mut ptr, builtin_cost_segment_start)?;

    let program_extra_data_length = 2;
    Ok(program_extra_data_length)
}

fn initialize_execution_context<'a>(
    call: ExecutableCallEntryPoint,
    // compiled_class: &'a CompiledClassV1,
    hints: &'a HashMap<String, Hint>,
    program: &Program,
    entry_point: EntryPointV1,
    state: &'a mut dyn State,
    context: &'a mut EntryPointExecutionContext,
) -> std::result::Result<VmExecutionContext<'a>, PreExecutionError> {
    // let entry_point = compiled_class.get_entry_point(&call)?;

    // Instantiate Cairo runner.
    let proof_mode = false;
    let trace_enabled = true;
    let mut runner = CairoRunner::new(
        // &compiled_class.0.program,
        program,
        LayoutName::all_cairo,
        proof_mode,
        trace_enabled,
    )?;

    runner.initialize_function_runner_cairo_1(&entry_point.builtins)?;
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
        entry_point,
        program_extra_data_length,
    })
}

/// Fills the holes after running the entry point.
/// Currently only fills the holes in the rc96 segment.
fn maybe_fill_holes(
    entry_point: EntryPointV1,
    runner: &mut CairoRunner,
) -> Result<(), EntryPointExecutionError> {
    let Some(rc96_offset) = entry_point
        .builtins
        .iter()
        .rev()
        .position(|name| *name == BuiltinName::range_check96)
    else {
        return Ok(());
    };
    let rc96_builtin_runner = runner
        .vm
        .get_builtin_runners()
        .iter()
        .find_map(|builtin| {
            if let BuiltinRunner::RangeCheck96(rc96_builtin_runner) = builtin {
                Some(rc96_builtin_runner)
            } else {
                None
            }
        })
        .expect("RangeCheck96 builtin runner not found.");

    // 'EntryPointReturnValues' is returned after the implicits and its size is 5,
    // So the last implicit is at offset 5 + 1.
    const IMPLICITS_OFFSET: usize = 6;
    let rc_96_stop_ptr = (runner.vm.get_ap() - (IMPLICITS_OFFSET + rc96_offset))
        .map_err(|err| CairoRunError::VirtualMachine(VirtualMachineError::Math(err)))?;

    let rc96_base = rc96_builtin_runner.base();
    let rc96_segment: isize = rc96_base
        .try_into()
        .expect("Builtin segment index must fit in isize.");

    let Relocatable {
        segment_index: rc96_stop_segment,
        offset: stop_offset,
    } = runner
        .vm
        .get_relocatable(rc_96_stop_ptr)
        .map_err(CairoRunError::MemoryError)?;
    assert_eq!(rc96_stop_segment, rc96_segment);

    // Update `segment_used_sizes` to include the holes.
    runner
        .vm
        .segments
        .segment_used_sizes
        .as_mut()
        .expect("Segments used sizes should be calculated at this point")[rc96_base] = stop_offset;

    for offset in 0..stop_offset {
        match runner.vm.insert_value(
            Relocatable {
                segment_index: rc96_segment,
                offset,
            },
            Felt::zero(),
        ) {
            // If the value is already set, ignore the error.
            Ok(()) | Err(MemoryError::InconsistentMemory(_)) => {}
            Err(err) => panic!("Unexpected error when filling holes: {err}."),
        }
    }

    Ok(())
}

pub fn run_entry_point(
    runner: &mut CairoRunner,
    hint_processor: &mut dyn HintProcessor,
    entry_point: EntryPointV1,
    args: Args,
    program_segment_size: usize,
) -> EntryPointExecutionResult<()> {
    // Note that we run `verify_secure_runner` manually after filling the holes in the rc96 segment.
    let verify_secure = false;
    let args: Vec<&CairoArg> = args.iter().collect();
    runner.run_from_entrypoint(
        entry_point.pc(),
        &args,
        verify_secure,
        Some(program_segment_size),
        hint_processor,
    )?;

    maybe_fill_holes(entry_point, runner)?;

    verify_secure_runner(runner, false, Some(program_segment_size))
        .map_err(CairoRunError::VirtualMachine)?;

    Ok(())
}

pub fn prepare_call_arguments(
    call: &CallEntryPoint,
    runner: &mut CairoRunner,
    initial_syscall_ptr: Relocatable,
    read_only_segments: &mut ReadOnlySegments,
    entrypoint: &EntryPointV1,
    entry_point_initial_budget: u64,
) -> std::result::Result<Args, PreExecutionError> {
    let mut args: Args = vec![];

    // Push builtins.
    for builtin_name in &entrypoint.builtins {
        if let Some(builtin) = runner
            .vm
            .get_builtin_runners()
            .iter()
            .find(|builtin| builtin.name() == *builtin_name)
        {
            args.extend(builtin.initial_stack().into_iter().map(CairoArg::Single));
            continue;
        }
        if builtin_name == &BuiltinName::segment_arena {
            let segment_arena = runner.vm.add_memory_segment();

            // Write into segment_arena.
            let mut ptr = segment_arena;
            let info_segment = runner.vm.add_memory_segment();
            let n_constructed = Felt::default();
            let n_destructed = Felt::default();
            write_maybe_relocatable(&mut runner.vm, &mut ptr, info_segment)?;
            write_felt(&mut runner.vm, &mut ptr, n_constructed)?;
            write_felt(&mut runner.vm, &mut ptr, n_destructed)?;

            args.push(CairoArg::Single(MaybeRelocatable::from(ptr)));
            continue;
        }
        return Err(PreExecutionError::InvalidBuiltin(*builtin_name));
    }
    // Pre-charge entry point's initial budget to ensure sufficient gas for executing a minimal
    // entry point code. When redepositing is used, the entry point is aware of this pre-charge
    // and adjusts the gas counter accordingly if a smaller amount of gas is required.
    let call_initial_gas = call
        .initial_gas
        .checked_sub(entry_point_initial_budget)
        .ok_or(PreExecutionError::InsufficientEntryPointGas)?;
    // Push gas counter.
    // dbg!(&call_initial_gas);
    args.push(CairoArg::Single(MaybeRelocatable::from(Felt::from(
        call_initial_gas,
    ))));
    // Push syscall ptr.
    args.push(CairoArg::Single(MaybeRelocatable::from(
        initial_syscall_ptr,
    )));

    // Prepare calldata arguments.
    // let calldata = &call.calldata.0;
    // let calldata: Vec<MaybeRelocatable> =
    //     calldata.iter().map(|&arg| MaybeRelocatable::from(arg)).collect();
    //
    // let calldata_start_ptr = read_only_segments.allocate(&mut runner.vm, &calldata)?;
    // let calldata_end_ptr = MaybeRelocatable::from((calldata_start_ptr + calldata.len())?);
    // args.push(CairoArg::Single(MaybeRelocatable::from(calldata_start_ptr)));
    // args.push(CairoArg::Single(calldata_end_ptr));

    Ok(args)
}

pub fn finalize_execution(
    runner: &mut CairoRunner,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    n_total_args: usize,
    program_extra_data_length: usize,
    tracked_resource: TrackedResource,
) -> std::result::Result<CallInfo, PostExecutionError> {
    // Close memory holes in segments (OS code touches those memory cells, we simulate it).
    let program_start_ptr = runner
        .program_base
        .expect("The `program_base` field should be initialized after running the entry point.");
    let program_end_ptr = (program_start_ptr + runner.get_program().data_len())?;
    runner
        .vm
        .mark_address_range_as_accessed(program_end_ptr, program_extra_data_length)?;

    let initial_fp = runner
        .get_initial_fp()
        .expect("The `initial_fp` field should be initialized after running the entry point.");
    // When execution starts the stack holds the EP arguments + [ret_fp, ret_pc].
    let args_ptr = (initial_fp - (n_total_args + 2))?;
    runner
        .vm
        .mark_address_range_as_accessed(args_ptr, n_total_args)?;
    syscall_handler
        .read_only_segments
        .mark_as_accessed(runner)?;

    let call_result = get_call_result(&runner, &syscall_handler, &tracked_resource)?;

    let vm_resources_without_inner_calls = match tracked_resource {
        TrackedResource::CairoSteps => {
            // Take into account the resources of the current call, without inner calls.
            // Has to happen after marking holes in segments as accessed.
            let mut vm_resources_without_inner_calls = runner
                .get_execution_resources()
                .map_err(VirtualMachineError::RunnerError)?
                .filter_unused_builtins();
            let versioned_constants = syscall_handler.base.context.versioned_constants();
            if versioned_constants.segment_arena_cells {
                vm_resources_without_inner_calls
                    .builtin_instance_counter
                    .get_mut(&BuiltinName::segment_arena)
                    .map_or_else(|| {}, |val| *val *= SEGMENT_ARENA_BUILTIN_SIZE);
            }
            // Take into account the syscall resources of the current call.
            vm_resources_without_inner_calls += &versioned_constants
                .get_additional_os_syscall_resources(&syscall_handler.syscalls_usage);
            vm_resources_without_inner_calls
        }
        TrackedResource::SierraGas => ExecutionResources::default(),
    };

    syscall_handler.finalize();

    let vm_resources = &vm_resources_without_inner_calls
        + &CallInfo::summarize_vm_resources(syscall_handler.base.inner_calls.iter());
    let syscall_handler_base = &syscall_handler.base;
    Ok(CallInfo {
        call: syscall_handler_base.call.clone().into(),
        execution: CallExecution {
            retdata: call_result.retdata,
            events: syscall_handler_base.events.clone(),
            l2_to_l1_messages: syscall_handler_base.l2_to_l1_messages.clone(),
            failed: call_result.failed,
            gas_consumed: call_result.gas_consumed,
        },
        inner_calls: syscall_handler_base.inner_calls.clone(),
        tracked_resource,
        resources: vm_resources,
        storage_read_values: syscall_handler_base.read_values.clone(),
        accessed_storage_keys: syscall_handler_base.accessed_keys.clone(),
        read_class_hash_values: syscall_handler_base.read_class_hash_values.clone(),
        accessed_contract_addresses: syscall_handler_base.accessed_contract_addresses.clone(),
    })
}

fn get_call_result(
    runner: &CairoRunner,
    syscall_handler: &SyscallHintProcessor<'_>,
    tracked_resource: &TrackedResource,
) -> std::result::Result<CallResult, PostExecutionError> {
    let return_result = runner.vm.get_return_values(5)?;
    // Corresponds to the Cairo 1.0 enum:
    // enum PanicResult<Array::<felt>> { Ok: Array::<felt>, Err: Array::<felt>, }.
    let [failure_flag, retdata_start, retdata_end]: &[MaybeRelocatable; 3] = (&return_result[2..])
        .try_into()
        .expect("Return values must be of size 3.");

    let failed = if *failure_flag == MaybeRelocatable::from(0) {
        false
    } else if *failure_flag == MaybeRelocatable::from(1) {
        true
    } else {
        return Err(PostExecutionError::MalformedReturnData {
            error_message: "Failure flag expected to be either 0 or 1.".to_string(),
        });
    };

    let retdata_size = retdata_end.sub(retdata_start)?;
    // TODO(spapini): Validate implicits.

    let gas = &return_result[0];
    let MaybeRelocatable::Int(gas) = gas else {
        return Err(PostExecutionError::MalformedReturnData {
            error_message: "Error extracting return data.".to_string(),
        });
    };
    let gas = gas
        .to_u64()
        .ok_or(PostExecutionError::MalformedReturnData {
            error_message: format!("Unexpected remaining gas: {gas}."),
        })?;

    if gas > syscall_handler.base.call.initial_gas {
        return Err(PostExecutionError::MalformedReturnData {
            error_message: format!("Unexpected remaining gas: {gas}."),
        });
    }

    let gas_consumed = match tracked_resource {
        // Do not count Sierra gas in CairoSteps mode.
        TrackedResource::CairoSteps => 0,
        TrackedResource::SierraGas => syscall_handler.base.call.initial_gas - gas,
    };
    Ok(CallResult {
        failed,
        retdata: read_execution_retdata(runner, retdata_size, retdata_start)?,
        gas_consumed,
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

#[expect(clippy::too_many_lines)]
pub fn run_test_case(
    case: &TestCaseWithResolvedConfig,
    casm_program: &AssembledProgramWithDebugInfo,
    runtime_config: &RuntimeConfig,
    fuzzer_rng: Option<Arc<Mutex<StdRng>>>,
) -> Result<RunResult> {
    // dbg!(&case.test_details.parameter_types);
    // dbg!(&case.test_details.return_types);

    let sierra_instruction_idx = case.test_details.sierra_entry_point_statement_idx;
    let casm_entry_point_offset = casm_program.debug_info[sierra_instruction_idx].0;

    let builtins = process_builtins(&case.test_details.parameter_types);

    let assembled_program = &casm_program.assembled_cairo_program;
    let hints_dict = hints_to_params(&assembled_program);
    let data: Vec<MaybeRelocatable> = assembled_program
        .bytecode
        .iter()
        .map(Felt::from)
        .map(MaybeRelocatable::from)
        .collect();
    // let data_len = data.len();
    let program = Program::new(
        builtins.clone(),
        data,
        Some(0),
        hints_dict,
        ReferenceManager {
            references: Vec::new(),
        },
        HashMap::new(),
        vec![],
        None,
    )
    .unwrap();

    // println!("Program\n{program}");

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

    let call = build_test_entry_point();
    let entry_point = EntryPointV1 {
        selector: call.entry_point_selector,
        offset: EntryPointOffset(casm_entry_point_offset),
        builtins: builtins.clone(),
    };

    let string_to_hint = hints_by_representation(&assembled_program);

    let VmExecutionContext {
        mut runner,
        syscall_handler,
        initial_syscall_ptr,
        entry_point,
        program_extra_data_length,
    } = initialize_execution_context(
        call.clone(),
        &string_to_hint,
        &program,
        entry_point,
        &mut cached_state,
        &mut context,
    )?;

    // let mut cached_state = CachedState::new(state_reader);
    // let string_to_hint = hints_by_representation(&assembled_program);
    //
    // let syscall_handler = build_syscall_handler(
    //     &mut cached_state,
    //     &string_to_hint,
    //     &mut context,
    //     &case.test_details.parameter_types,
    //     builtins.len(),
    // );

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
            // Max gas is no longer set by `create_entry_code_from_params`
            // Instead, call to `ExternalHint::WriteRunParam` is added by it, and we need to
            // store the gas value to be read by logic handling the hint
            // TODO(#2966) we should subtract initial cost of the function from this value to be more exact.
            //  But as a workaround it should be good enough.
            user_args: vec![vec![Arg::Value(Felt::from(i64::MAX as u64))]],
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

    // let entry_point_initial_budget = context.gas_costs().base.entry_point_initial_budget;
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
        // &mut syscall_handler.read_only_segments,
        &mut forge_runtime
            .extended_runtime
            .extended_runtime
            .extended_runtime
            .hint_handler
            .read_only_segments,
        &entry_point,
        entry_point_initial_budget,
    )?;
    // dbg!(&entry_point.builtins);
    // dbg!(&call.initial_gas);
    // dbg!(&entry_point_initial_budget);
    // dbg!(&args);

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

    // dbg!(&result);

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

    println!("Used resources: {:?}", used_resources);
    println!("Gas used: {:?}", gas);

    // TODO change this
    // bail!("Failed")

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

    // Ok(RunResultWithInfo {
    //     run_result: Ok(RunResult {
    //         gas_counter: None,
    //         memory: vec![],
    //         value: if result.execution.failed {
    //             RunResultValue::Panic(result.execution.retdata.0)
    //         } else {
    //             RunResultValue::Success(result.execution.retdata.0)
    //         },
    //         used_resources: used_resources.execution_resources.clone(),
    //         profiling_info: None,
    //     }),
    //     gas_used: gas,
    //     used_resources,
    //     call_trace: call_trace_ref,
    //     encountered_errors,
    //     fuzzer_args,
    // })
}

// #[expect(clippy::too_many_lines)]
// pub fn run_test_case2(
//     case: &TestCaseWithResolvedConfig,
//     casm_program: &AssembledProgramWithDebugInfo,
//     runtime_config: &RuntimeConfig,
//     fuzzer_rng: Option<Arc<Mutex<StdRng>>>,
// ) -> Result<RunResultWithInfo> {
//     ensure!(
//         case.config
//             .available_gas
//             .as_ref()
//             .is_none_or(|gas| !gas.is_zero()),
//         "\n\t`available_gas` attribute was incorrectly configured. Make sure you use scarb >= 2.4.4\n"
//     );
//
//     let (entry_code, builtins) = create_entry_code(&case.test_details, casm_program);
//
//     let assembled_program = get_assembled_program(casm_program, entry_code);
//
//     let string_to_hint = hints_by_representation(&assembled_program);
//     let hints_dict = hints_to_params(&assembled_program);
//
//     let mut state_reader = ExtendedStateReader {
//         dict_state_reader: cheatnet_constants::build_testing_state(),
//         fork_state_reader: get_fork_state_reader(
//             runtime_config.cache_dir,
//             case.config.fork_config.as_ref(),
//         )?,
//     };
//     let block_info = state_reader.get_block_info()?;
//     let chain_id = state_reader.get_chain_id()?;
//     let tracked_resource = TrackedResource::from(runtime_config.tracked_resource);
//
//     let mut context = build_context(&block_info, chain_id, &tracked_resource);
//
//     if let Some(max_n_steps) = runtime_config.max_n_steps {
//         set_max_steps(&mut context, max_n_steps);
//     }
//     let mut cached_state = CachedState::new(state_reader);
//     let syscall_handler = build_syscall_handler(
//         &mut cached_state,
//         &string_to_hint,
//         &mut context,
//         &case.test_details.parameter_types,
//         builtins.len(),
//     );
//
//     let mut cheatnet_state = CheatnetState {
//         block_info,
//         ..Default::default()
//     };
//     cheatnet_state.trace_data.is_vm_trace_needed = runtime_config.is_vm_trace_needed;
//
//     let cheatable_runtime = ExtendedRuntime {
//         extension: CheatableStarknetRuntimeExtension {
//             cheatnet_state: &mut cheatnet_state,
//         },
//         extended_runtime: StarknetRuntime {
//             hint_handler: syscall_handler,
//             // Max gas is no longer set by `create_entry_code_from_params`
//             // Instead, call to `ExternalHint::WriteRunParam` is added by it, and we need to
//             // store the gas value to be read by logic handling the hint
//             // TODO(#2966) we should subtract initial cost of the function from this value to be more exact.
//             //  But as a workaround it should be good enough.
//             user_args: vec![vec![Arg::Value(Felt::from(i64::MAX as u64))]],
//         },
//     };
//
//     let call_to_blockifier_runtime = ExtendedRuntime {
//         extension: CallToBlockifierExtension {
//             lifetime: &PhantomData,
//         },
//         extended_runtime: cheatable_runtime,
//     };
//     let forge_extension = ForgeExtension {
//         environment_variables: runtime_config.environment_variables,
//         contracts_data: runtime_config.contracts_data,
//         fuzzer_rng,
//     };
//
//     let mut forge_runtime = ExtendedRuntime {
//         extension: forge_extension,
//         extended_runtime: call_to_blockifier_runtime,
//     };
//
//     let run_result =
//         match run_assembled_program(&assembled_program, builtins, hints_dict, &mut forge_runtime) {
//             Ok(mut runner) => {
//                 let vm_resources_without_inner_calls = runner
//                     .get_execution_resources()
//                     .expect("Execution resources missing")
//                     .filter_unused_builtins();
//                 add_resources_to_top_call(
//                     &mut forge_runtime,
//                     &vm_resources_without_inner_calls,
//                     &tracked_resource,
//                 );
//
//                 let ap = runner.relocated_trace.as_ref().unwrap().last().unwrap().ap;
//
//                 let results_data = get_results_data(
//                     &case.test_details.return_types,
//                     &runner.relocated_memory,
//                     ap,
//                 );
//                 assert_eq!(results_data.len(), 1);
//
//                 let (_, values) = results_data[0].clone();
//                 let value = SierraCasmRunner::handle_main_return_value(
//                     // Here we assume that all test either panic or do not return any value
//                     // This is true for all test right now, but in case it changes
//                     // this logic will need to be updated
//                     Some(0),
//                     values,
//                     &runner.relocated_memory,
//                 );
//
//                 update_top_call_vm_trace(&mut forge_runtime, &mut runner);
//
//                 Ok(value)
//             }
//             Err(err) => Err(err),
//         };
//
//     let encountered_errors = forge_runtime
//         .extended_runtime
//         .extended_runtime
//         .extension
//         .cheatnet_state
//         .encountered_errors
//         .clone();
//
//     let call_trace = get_call_trace_ref(&mut forge_runtime);
//
//     update_top_call_resources(&mut forge_runtime, &tracked_resource);
//     update_top_call_l1_resources(&mut forge_runtime);
//
//     let fuzzer_args = forge_runtime
//         .extended_runtime
//         .extended_runtime
//         .extension
//         .cheatnet_state
//         .fuzzer_args
//         .clone();
//
//     let transaction_context = get_context(&forge_runtime).tx_context.clone();
//     let used_resources =
//         get_all_used_resources(forge_runtime, &transaction_context, tracked_resource);
//     let gas_used = calculate_used_gas(
//         &transaction_context,
//         &mut cached_state,
//         used_resources.clone(),
//     )?;
//
//     Ok(match run_result {
//         Ok(result) => RunResult::Completed(Box::from(RunCompleted {
//             status: result.into(),
//             call_trace,
//             gas_used,
//             used_resources,
//             encountered_errors,
//             fuzzer_args,
//         })),
//         Err(error) => RunResult::Error(RunError {
//             error,
//             call_trace,
//             encountered_errors,
//             fuzzer_args,
//         }),
//     })
// }

// TODO(#2958) Remove copied code
// Copied and modified from https://github.com/starkware-libs/cairo/blob/a8da296d7d03f19af3bdb0e7ae17637e66192e4b/crates/cairo-lang-runner/src/lib.rs#L543
#[allow(clippy::cast_sign_loss)]
#[must_use]
pub fn get_results_data(
    return_types: &[(GenericTypeId, i16)],
    cells: &[Option<Felt252>],
    mut ap: usize,
) -> Vec<(GenericTypeId, Vec<Felt252>)> {
    let mut results_data = vec![];
    for (ty, ty_size) in return_types.iter().rev() {
        let size = *ty_size as usize;
        let values: Vec<Felt252> = ((ap - size)..ap)
            .map(|index| cells[index].unwrap())
            .collect();
        ap -= size;
        results_data.push((ty.clone(), values));
    }

    // Handling implicits.
    results_data.retain_mut(|(ty, values)| {
        let generic_ty = ty;
        if *generic_ty == GasBuiltinType::ID {
            values.remove(0);
            assert!(values.is_empty());
            false
        } else {
            // region: Modified code
            let non_user_types: UnorderedHashSet<GenericTypeId> = UnorderedHashSet::from_iter([
                AddModType::ID,
                BitwiseType::ID,
                GasBuiltinType::ID,
                EcOpType::ID,
                MulModType::ID,
                PedersenType::ID,
                PoseidonType::ID,
                RangeCheck96Type::ID,
                RangeCheckType::ID,
                SegmentArenaType::ID,
                SystemType::ID,
            ]);
            !non_user_types.contains(generic_ty)
            // endregion
        }
    });

    results_data
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
