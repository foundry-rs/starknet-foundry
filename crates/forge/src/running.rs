use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, bail, Result};
use blockifier::execution::common_hints::ExecutionMode;
use blockifier::execution::entry_point::{
    CallEntryPoint, CallType, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::execution::execution_utils::ReadOnlySegments;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::cached_state::CachedState;
use blockifier::state::state_api::State;
use cairo_felt::Felt252;
use cairo_vm::serde::deserialize_program::HintParams;
use cairo_vm::types::relocatable::Relocatable;
use cheatnet::execution::cheatable_syscall_handler::CheatableSyscallHandler;
use itertools::chain;

use cairo_lang_casm::hints::Hint;
use cairo_lang_casm::instructions::Instruction;
use cairo_lang_runner::casm_run::hint_to_hint_params;
use cairo_lang_runner::{Arg, RunnerError};
use cairo_lang_runner::{RunResult, SierraCasmRunner};
use camino::Utf8Path;
use cheatnet::constants as cheatnet_constants;
use cheatnet::execution::contract_execution_syscall_handler::ContractExecutionSyscallHandler;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::state::{BlockInfoReader, CheatnetBlockInfo, CheatnetState, ExtendedStateReader};
use starknet::core::types::BlockTag::Latest;
use starknet::core::types::{BlockId, MaybePendingBlockWithTxHashes};
use starknet::core::utils::get_selector_from_name;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use starknet_api::block::BlockNumber;
use starknet_api::core::PatriciaKey;
use starknet_api::core::{ContractAddress, EntryPointSelector};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkHash;
use starknet_api::patricia_key;
use starknet_api::transaction::Calldata;
use test_collector::TestCase;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use url::Url;

use crate::test_case_summary::TestCaseSummary;

use crate::collecting::{TestCaseRunnable, ValidatedForkConfig};
use crate::test_execution_syscall_handler::TestExecutionSyscallHandler;
use crate::{RunnerConfig, RunnerParams, CACHE_DIR};

use crate::test_execution_syscall_handler::TestExecutionState;

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

pub(crate) fn run_test(
    case: Arc<TestCaseRunnable>,
    runner: Arc<SierraCasmRunner>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    send: Sender<()>,
    send_shut_down: Sender<()>,
) -> JoinHandle<Result<TestCaseSummary>> {
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
            &runner,
            &runner_config,
            &runner_params,
            &send_shut_down,
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
    runner: Arc<SierraCasmRunner>,
    runner_config: Arc<RunnerConfig>,
    runner_params: Arc<RunnerParams>,
    send: Sender<()>,
    fuzzing_send: Sender<()>,
    send_shut_down: Sender<()>,
) -> JoinHandle<Result<TestCaseSummary>> {
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
            &runner,
            &runner_config,
            &runner_params,
            &send_shut_down,
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

fn build_context(block_info: CheatnetBlockInfo) -> EntryPointExecutionContext {
    let block_context = cheatnet_constants::build_block_context(block_info);
    let account_context = cheatnet_constants::build_transaction_context();
    EntryPointExecutionContext::new(&block_context, &account_context, ExecutionMode::Execute)
}

fn build_syscall_handler<'a>(
    blockifier_state: &'a mut dyn State,
    string_to_hint: &'a HashMap<String, Hint>,
    execution_resources: &'a mut ExecutionResources,
    context: &'a mut EntryPointExecutionContext,
) -> SyscallHintProcessor<'a> {
    let test_selector = get_selector_from_name("TEST_CONTRACT_SELECTOR").unwrap();
    let entry_point_selector =
        EntryPointSelector(StarkHash::new(test_selector.to_bytes_be()).unwrap());
    let entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(ContractAddress(patricia_key!(
            cheatnet_constants::TEST_ADDRESS
        ))),
        entry_point_type: EntryPointType::External,
        entry_point_selector,
        calldata: Calldata(Arc::new(vec![])),
        storage_address: ContractAddress(patricia_key!(cheatnet_constants::TEST_ADDRESS)),
        caller_address: ContractAddress::default(),
        call_type: CallType::Call,
        initial_gas: u64::MAX,
    };

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

pub(crate) struct ForkInfo {
    pub(crate) latest_block_number: Option<BlockNumber>,
}

pub(crate) struct RunResultWithInfo {
    pub(crate) run_result: Result<RunResult, RunnerError>,
    pub(crate) fork_info: ForkInfo,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_test_case(
    args: Vec<Felt252>,
    case: &TestCaseRunnable,
    runner: &SierraCasmRunner,
    runner_config: &Arc<RunnerConfig>,
    runner_params: &Arc<RunnerParams>,
    _send_shut_down: &Sender<()>,
) -> Result<RunResultWithInfo> {
    let available_gas = if let Some(available_gas) = &case.available_gas {
        Some(*available_gas)
    } else {
        Some(usize::MAX)
    };
    let func = runner.find_function(case.name.as_str()).unwrap();
    let initial_gas = runner
        .get_initial_available_gas(func, available_gas)
        .unwrap();
    let runner_args: Vec<Arg> = args.into_iter().map(Arg::Value).collect();

    let (entry_code, builtins) = runner
        .create_entry_code(func, &runner_args, initial_gas)
        .unwrap();
    let footer = runner.create_code_footer();
    let instructions = chain!(
        entry_code.iter(),
        runner.get_casm_program().instructions.iter(),
        footer.iter()
    );
    let (hints_dict, string_to_hint) = build_hints_dict(instructions.clone());

    let mut state_reader = ExtendedStateReader {
        dict_state_reader: cheatnet_constants::build_testing_state(
            &runner_params.predeployed_contracts,
        ),
        fork_state_reader: get_fork_state_reader(&runner_config.workspace_root, &case.fork_config)?,
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
    let cheatable_syscall_handler =
        CheatableSyscallHandler::wrap(syscall_handler, &mut cheatnet_state);
    let contract_execution_syscall_handler =
        ContractExecutionSyscallHandler::wrap(cheatable_syscall_handler);

    let mut test_execution_state = TestExecutionState {
        environment_variables: &runner_params.environment_variables,
        contracts: &runner_params.contracts,
    };
    let mut test_execution_syscall_handler = TestExecutionSyscallHandler::wrap(
        contract_execution_syscall_handler,
        &mut test_execution_state,
        &string_to_hint,
    );

    let latest_block_number = if let Some(ValidatedForkConfig {
        url: _,
        block_id: BlockId::Tag(Latest),
    }) = &case.fork_config
    {
        Some(block_info.block_number)
    } else {
        None
    };

    let run_result = runner.run_function(
        func,
        &mut test_execution_syscall_handler,
        hints_dict,
        instructions,
        builtins,
    );

    Ok(RunResultWithInfo {
        run_result,
        fork_info: ForkInfo {
            latest_block_number,
        },
    })
}

fn extract_test_case_summary(
    run_result: Result<RunResultWithInfo>,
    case: &TestCase<ValidatedForkConfig>,
    args: Vec<Felt252>,
) -> Result<TestCaseSummary> {
    match run_result {
        Ok(result_with_info) => {
            match result_with_info.run_result {
                Ok(run_result) => Ok(TestCaseSummary::from_run_result_and_info(
                    run_result,
                    case,
                    args,
                    &result_with_info.fork_info,
                )),
                // CairoRunError comes from VirtualMachineError which may come from HintException that originates in TestExecutionSyscallHandler
                Err(RunnerError::CairoRunError(error)) => Ok(TestCaseSummary::Failed {
                    name: case.name.clone(),
                    msg: Some(format!(
                        "\n    {}\n",
                        error.to_string().replace(" Custom Hint Error: ", "\n    ")
                    )),
                    arguments: args,
                    fuzzing_statistic: None,
                    latest_block_number: result_with_info.fork_info.latest_block_number,
                }),
                Err(err) => bail!(err),
            }
        }
        // `ForkStateReader.get_block_info` and `get_fork_state_reader` may return an error
        Err(error) => Ok(TestCaseSummary::Failed {
            name: case.name.clone(),
            msg: Some(error.to_string()),
            arguments: args,
            fuzzing_statistic: None,
            latest_block_number: None,
        }),
    }
}

fn get_fork_state_reader(
    workspace_root: &Utf8Path,
    fork_config: &Option<ValidatedForkConfig>,
) -> Result<Option<ForkStateReader>> {
    match fork_config {
        Some(ValidatedForkConfig { url, mut block_id }) => {
            if let BlockId::Tag(Latest) = block_id {
                block_id = get_latest_block_number(url)?;
            }
            Ok(Some(ForkStateReader::new(
                url.clone(),
                block_id,
                Some(workspace_root.join(CACHE_DIR).as_ref()),
            )))
        }
        None => Ok(None),
    }
}

fn get_latest_block_number(url: &Url) -> Result<BlockId> {
    let client = JsonRpcClient::new(HttpTransport::new(url.clone()));
    let runtime = Runtime::new().expect("Could not instantiate Runtime");

    match runtime.block_on(client.get_block_with_tx_hashes(BlockId::Tag(Latest))) {
        Ok(MaybePendingBlockWithTxHashes::Block(block)) => Ok(BlockId::Number(block.block_number)),
        _ => Err(anyhow!("Could not get the latest block number".to_string())),
    }
}
