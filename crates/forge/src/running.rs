use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use blockifier::execution::entry_point::{
    CallEntryPoint, CallType, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::execution::execution_utils::ReadOnlySegments;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::cached_state::{CachedState, GlobalContractCache};
use blockifier::state::state_api::State;
use cairo_felt::Felt252;
use cairo_vm::serde::deserialize_program::HintParams;
use cairo_vm::types::relocatable::Relocatable;
use cheatnet::constants::{
    build_block_context, build_testing_state, build_transaction_context, TEST_ADDRESS,
};
use itertools::chain;

use cairo_lang_casm::hints::Hint;
use cairo_lang_casm::instructions::Instruction;
use cairo_lang_runner::casm_run::hint_to_hint_params;
use cairo_lang_runner::SierraCasmRunner;
use cairo_lang_runner::{Arg, RunnerError};
use camino::Utf8PathBuf;
use cheatnet::execution::syscalls::CheatableSyscallHandler;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::state::{CheatnetState, ExtendedStateReader};
use conversions::StarknetConversions;
use starknet::core::types::{BlockId, BlockTag};
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::PatriciaKey;
use starknet_api::core::{ContractAddress, EntryPointSelector};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkHash;
use starknet_api::patricia_key;
use starknet_api::transaction::Calldata;
use test_collector::{ForkConfig, TestCase};

use crate::scarb::{ForkTarget, StarknetContractArtifacts};
use crate::test_case_summary::TestCaseSummary;
use crate::test_execution_syscall_handler::{TestExecutionState, TestExecutionSyscallHandler};

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

fn build_context() -> EntryPointExecutionContext {
    let block_context = build_block_context();
    let account_context = build_transaction_context();
    EntryPointExecutionContext::new(
        block_context.clone(),
        account_context,
        block_context.invoke_tx_max_n_steps.try_into().unwrap(),
    )
}

fn build_syscall_handler<'a>(
    blockifier_state: &'a mut dyn State,
    string_to_hint: &'a HashMap<String, Hint>,
    execution_resources: &'a mut ExecutionResources,
    context: &'a mut EntryPointExecutionContext,
) -> Result<SyscallHintProcessor<'a>> {
    let test_selector = get_selector_from_name("TEST_CONTRACT_SELECTOR").unwrap();
    let entry_point_selector = EntryPointSelector(StarkHash::new(test_selector.to_bytes_be())?);
    let entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(ContractAddress(patricia_key!(TEST_ADDRESS))),
        entry_point_type: EntryPointType::External,
        entry_point_selector,
        calldata: Calldata(Arc::new(vec![])),
        storage_address: ContractAddress(patricia_key!(TEST_ADDRESS)),
        caller_address: ContractAddress::default(),
        call_type: CallType::Call,
        initial_gas: u64::MAX,
    };

    let syscall_handler = SyscallHintProcessor::new(
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
    );
    Ok(syscall_handler)
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_test_case(
    package_root: &Utf8PathBuf,
    runner: &SierraCasmRunner,
    case: &TestCase,
    fork_targets: &[ForkTarget],
    contracts: &HashMap<String, StarknetContractArtifacts>,
    predeployed_contracts: &Utf8PathBuf,
    args: Vec<Felt252>,
    environment_variables: &HashMap<String, String>,
) -> Result<TestCaseSummary> {
    let available_gas = if let Some(available_gas) = &case.available_gas {
        Some(*available_gas)
    } else {
        Some(usize::MAX)
    };
    let func = runner.find_function(case.name.as_str())?;
    let initial_gas = runner.get_initial_available_gas(func, available_gas)?;
    let runner_args: Vec<Arg> = args.clone().into_iter().map(Arg::Value).collect();

    let (entry_code, builtins) = runner.create_entry_code(func, &runner_args, initial_gas)?;
    let footer = runner.create_code_footer();
    let instructions = chain!(
        entry_code.iter(),
        runner.get_casm_program().instructions.iter(),
        footer.iter()
    );
    let (hints_dict, string_to_hint) = build_hints_dict(instructions.clone());

    let state_reader = ExtendedStateReader {
        dict_state_reader: build_testing_state(predeployed_contracts),
        fork_state_reader: get_fork_state_reader(package_root, fork_targets, &case.fork_config),
    };
    let mut context = build_context();
    let mut execution_resources = ExecutionResources::default();
    let mut blockifier_state = CachedState::new(state_reader, GlobalContractCache::default());
    let syscall_handler = build_syscall_handler(
        &mut blockifier_state,
        &string_to_hint,
        &mut execution_resources,
        &mut context,
    )?;

    let mut cheatnet_state = CheatnetState::default();
    let cheatable_syscall_handler =
        CheatableSyscallHandler::new(syscall_handler, &mut cheatnet_state);

    let mut test_execution_state = TestExecutionState {
        environment_variables,
        contracts,
    };
    let mut test_execution_syscall_handler = TestExecutionSyscallHandler::new(
        cheatable_syscall_handler,
        &mut test_execution_state,
        &string_to_hint,
    );

    match runner.run_function(
        runner.find_function(case.name.as_str())?,
        &mut test_execution_syscall_handler,
        hints_dict,
        instructions,
        builtins,
    ) {
        Ok(result) => Ok(TestCaseSummary::from_run_result(result, case, args)),

        // CairoRunError comes from VirtualMachineError which may come from HintException that originates in the cheatcode processor
        Err(RunnerError::CairoRunError(error)) => Ok(TestCaseSummary::Failed {
            name: case.name.clone(),
            run_result: None,
            msg: Some(format!(
                "\n    {}\n",
                error.to_string().replace(" Custom Hint Error: ", "\n    ")
            )),
            arguments: args,
        }),

        Err(err) => Err(err.into()),
    }
}

fn get_fork_state_reader(
    package_root: &Utf8PathBuf,
    fork_targets: &[ForkTarget],
    fork_config: &Option<ForkConfig>,
) -> Option<ForkStateReader> {
    match &fork_config {
        Some(ForkConfig::Params(url, block_id)) => Some(ForkStateReader::new(
            url,
            *block_id,
            Some(package_root.join(".snfoundry_cache").as_ref()),
        )),
        Some(ForkConfig::Id(name)) => {
            find_params_and_build_fork_state_reader(package_root, fork_targets, name)
        }
        _ => None,
    }
}

fn find_params_and_build_fork_state_reader(
    package_root: &Utf8PathBuf,
    fork_targets: &[ForkTarget],
    fork_alias: &str,
) -> Option<ForkStateReader> {
    let fork = fork_targets.iter().find(|fork| fork.name == fork_alias);

    let block_id = fork?
        .block_id
        .iter()
        .map(|(id_type, value)| match id_type.as_str() {
            "number" => Some(BlockId::Number(value.parse().unwrap())),
            "hash" => Some(BlockId::Hash(value.to_field_element())),
            "tag" => match value.as_str() {
                "Latest" => Some(BlockId::Tag(BlockTag::Latest)),
                "Pending" => Some(BlockId::Tag(BlockTag::Pending)),
                _ => unreachable!(),
            },
            _ => unreachable!(),
        })
        .collect::<Vec<_>>();
    let [Some(block_id)] = block_id[..] else {
        return None;
    };

    Some(ForkStateReader::new(
        &fork?.url,
        block_id,
        Some(package_root.join(".snfoundry_cache").as_ref()),
    ))
}
