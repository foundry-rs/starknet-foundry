use crate::backtrace::add_backtrace_footer;
use crate::forge_config::{RuntimeConfig, TestRunnerConfig};
use crate::gas::calculate_used_gas;
use crate::package_tests::with_config_resolved::{ResolvedForkConfig, TestCaseWithResolvedConfig};
use crate::test_case_setup::{deploy_strk_token, is_strk_deployed};
use crate::test_case_summary::{Single, TestCaseSummary};
use anyhow::{Result, ensure};
use blockifier::execution::contract_class::TrackedResource;
use blockifier::execution::entry_point::EntryPointExecutionContext;
use blockifier::state::cached_state::CachedState;
use cairo_lang_runner::{Arg, RunResult, SierraCasmRunner};
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
use cairo_lang_utils::unordered_hash_set::UnorderedHashSet;
use cairo_vm::Felt252;
use cairo_vm::vm::errors::cairo_run_errors::CairoRunError;
use cairo_vm::vm::errors::vm_errors::VirtualMachineError;
use camino::{Utf8Path, Utf8PathBuf};
use casm::{get_assembled_program, run_assembled_program};
use cheatnet::constants as cheatnet_constants;
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
    BlockInfoReader, CallTrace, CheatnetState, EncounteredErrors, ExtendedStateReader,
};
use entry_code::create_entry_code;
use hints::{hints_by_representation, hints_to_params};
use rand::prelude::StdRng;
use runtime::starknet::context::{build_context, set_max_steps};
use runtime::{ExtendedRuntime, StarknetRuntime};
use starknet_api::execution_resources::GasVector;
use starknet_types_core::felt::Felt;
use std::cell::RefCell;
use std::default::Default;
use std::marker::PhantomData;
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
            vec![],
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
            vec![],
            &test_runner_config.contracts_data,
            &versioned_program_path,
        )
    })
}

pub struct RunResultWithInfo {
    pub(crate) run_result: Result<RunResult, Box<CairoRunError>>,
    pub(crate) call_trace: Rc<RefCell<CallTrace>>,
    pub(crate) gas_used: GasVector,
    pub(crate) used_resources: UsedResources,
    pub(crate) encountered_errors: EncounteredErrors,
    pub(crate) fuzzer_args: Vec<String>,
}

#[expect(clippy::too_many_lines)]
pub fn run_test_case(
    case: &TestCaseWithResolvedConfig,
    casm_program: &AssembledProgramWithDebugInfo,
    runtime_config: &RuntimeConfig,
    fuzzer_rng: Option<Arc<Mutex<StdRng>>>,
) -> Result<RunResultWithInfo> {
    ensure!(
        case.config
            .available_gas
            .as_ref()
            .is_none_or(|gas| !gas.is_zero()),
        "\n\t`available_gas` attribute was incorrectly configured. Make sure you use scarb >= 2.4.4\n"
    );

    let (entry_code, builtins) = create_entry_code(&case.test_details, casm_program);

    let assembled_program = get_assembled_program(casm_program, entry_code);

    let string_to_hint = hints_by_representation(&assembled_program);
    let hints_dict = hints_to_params(&assembled_program);

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

    let is_strk_deployed = is_strk_deployed(&mut state_reader);

    let mut cached_state = CachedState::new(state_reader);

    if !is_strk_deployed {
        deploy_strk_token(&mut cached_state);
    }

    let syscall_handler = build_syscall_handler(
        &mut cached_state,
        &string_to_hint,
        &mut context,
        &case.test_details.parameter_types,
        builtins.len(),
    );

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
            panic_traceback: None,
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

    let run_result =
        match run_assembled_program(&assembled_program, builtins, hints_dict, &mut forge_runtime) {
            Ok(mut runner) => {
                let vm_resources_without_inner_calls = runner
                    .get_execution_resources()
                    .expect("Execution resources missing")
                    .filter_unused_builtins();
                add_resources_to_top_call(
                    &mut forge_runtime,
                    &vm_resources_without_inner_calls,
                    &tracked_resource,
                );

                let ap = runner.relocated_trace.as_ref().unwrap().last().unwrap().ap;

                let (results_data, gas_counter) = get_results_data(
                    &case.test_details.return_types,
                    &runner.relocated_memory,
                    ap,
                );
                assert_eq!(results_data.len(), 1);

                let (_, values) = results_data[0].clone();
                let value = SierraCasmRunner::handle_main_return_value(
                    // Here we assume that all test either panic or do not return any value
                    // This is true for all test right now, but in case it changes
                    // this logic will need to be updated
                    Some(0),
                    values,
                    &runner.relocated_memory,
                );

                update_top_call_vm_trace(&mut forge_runtime, &mut runner);

                Ok((gas_counter, runner.relocated_memory, value))
            }
            Err(err) => Err(err),
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

    Ok(RunResultWithInfo {
        run_result: run_result.map(|(gas_counter, memory, value)| RunResult {
            used_resources: used_resources.execution_resources.clone(),
            gas_counter,
            memory,
            value,
            profiling_info: None,
        }),
        gas_used: gas,
        used_resources,
        call_trace: call_trace_ref,
        encountered_errors,
        fuzzer_args,
    })
}

// TODO(#2958) Remove copied code
// Copied and modified from https://github.com/starkware-libs/cairo/blob/a8da296d7d03f19af3bdb0e7ae17637e66192e4b/crates/cairo-lang-runner/src/lib.rs#L543
#[allow(clippy::cast_sign_loss)]
#[must_use]
pub fn get_results_data(
    return_types: &[(GenericTypeId, i16)],
    cells: &[Option<Felt252>],
    mut ap: usize,
) -> (Vec<(GenericTypeId, Vec<Felt252>)>, Option<Felt252>) {
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
    let mut gas_counter = None;
    results_data.retain_mut(|(ty, values)| {
        let generic_ty = ty;
        if *generic_ty == GasBuiltinType::ID {
            gas_counter = Some(values.remove(0));
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

    (results_data, gas_counter)
}

fn extract_test_case_summary(
    run_result: Result<RunResultWithInfo>,
    case: &TestCaseWithResolvedConfig,
    args: Vec<Felt>,
    contracts_data: &ContractsData,
    versioned_program_path: &Utf8Path,
) -> TestCaseSummary<Single> {
    match run_result {
        Ok(result_with_info) => {
            match result_with_info.run_result {
                Ok(run_result) => TestCaseSummary::from_run_result_and_info(
                    run_result,
                    case,
                    args,
                    result_with_info.fuzzer_args,
                    result_with_info.gas_used,
                    result_with_info.used_resources,
                    &result_with_info.call_trace,
                    &result_with_info.encountered_errors,
                    contracts_data,
                    versioned_program_path,
                ),
                // CairoRunError comes from VirtualMachineError which may come from HintException that originates in TestExecutionSyscallHandler
                Err(error) => {
                    let mut message = format!(
                        "\n    {}\n",
                        error.to_string().replace(" Custom Hint Error: ", "\n    ")
                    );
                    if let CairoRunError::VirtualMachine(VirtualMachineError::UnfinishedExecution) =
                        *error
                    {
                        message.push_str(
                                "\n    Suggestion: Consider using the flag `--max-n-steps` to increase allowed limit of steps",
                            );
                    }
                    TestCaseSummary::Failed {
                        name: case.name.clone(),
                        msg: Some(message).map(|msg| {
                            add_backtrace_footer(
                                msg,
                                contracts_data,
                                &result_with_info.encountered_errors,
                            )
                        }),
                        arguments: args,
                        fuzzer_args: result_with_info.fuzzer_args,
                        test_statistics: (),
                        debugging_trace: cfg!(feature = "debugging").then(|| {
                            debugging::Trace::new(
                                &result_with_info.call_trace.borrow(),
                                contracts_data,
                                case.name.clone(),
                            )
                        }),
                    }
                }
            }
        }
        // `ForkStateReader.get_block_info`, `get_fork_state_reader, `calculate_used_gas` may return an error
        // `available_gas` may be specified with Scarb ~2.4
        Err(error) => TestCaseSummary::Failed {
            name: case.name.clone(),
            msg: Some(error.to_string()),
            arguments: args,
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
