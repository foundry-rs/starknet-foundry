use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use blockifier::execution::entry_point::{
    CallEntryPoint, CallType, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::execution::execution_utils::ReadOnlySegments;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::cached_state::{CachedState, GlobalContractCache};
use cairo_felt::Felt252;
use cairo_vm::serde::deserialize_program::HintParams;
use cairo_vm::types::relocatable::Relocatable;
use cheatnet::constants::{build_block_context, build_testing_state, build_transaction_context};
use cheatnet::execution::syscalls::CheatableSyscallHandler;
use itertools::chain;

use cairo_lang_casm::hints::Hint;
use cairo_lang_casm::instructions::Instruction;
use cairo_lang_runner::casm_run::hint_to_hint_params;
use cairo_lang_runner::SierraCasmRunner;
use cairo_lang_runner::{Arg, RunnerError};
use cairo_vm::vm::runners::cairo_runner::RunResources;
use camino::Utf8PathBuf;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::state::{CheatnetState, ExtendedStateReader};
use starknet::core::types::BlockId;
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::PatriciaKey;
use starknet_api::core::{ContractAddress, EntryPointSelector};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkHash;
use starknet_api::patricia_key;
use starknet_api::transaction::Calldata;
use test_collector::{ForkConfig, TestCase};

use crate::cheatcodes_hint_processor::CheatcodesSyscallHandler;
use crate::scarb::{ForkTarget, StarknetContractArtifacts};
use crate::test_case_summary::{TestCaseSummary, Url};

// snforge_std/src/cheatcodes.cairo::test_address
const TEST_ADDRESS: &str = "0x01724987234973219347210837402";
const CACHE_FILE_NAME: &str = ".snfoundry_cache";

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

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_from_test_case(
    workspace_root: &Utf8PathBuf,
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
    let fork_params = extract_fork_params(fork_targets, &case.fork_config)?;

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

    // Losely inspired by crates/cheatnet/src/execution/cairo1_execution::execute_entry_point_call_cairo1
    let block_context = build_block_context();
    let account_context = build_transaction_context();
    let mut context = EntryPointExecutionContext::new(
        block_context.clone(),
        account_context,
        block_context.invoke_tx_max_n_steps.try_into().unwrap(),
    );
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

    let state_reader = ExtendedStateReader {
        dict_state_reader: build_testing_state(predeployed_contracts),
        fork_state_reader: get_fork_state_reader(workspace_root, fork_params.as_ref()),
    };
    let mut blockifier_state = CachedState::new(state_reader, GlobalContractCache::default());
    let mut execution_resources = ExecutionResources::default();

    let syscall_handler = SyscallHintProcessor::new(
        &mut blockifier_state,
        &mut execution_resources,
        &mut context,
        // This segment is created by SierraCasmRunner
        Relocatable {
            segment_index: 10,
            offset: 0,
        },
        entry_point,
        &string_to_hint,
        ReadOnlySegments::default(),
    );

    let cheatable_syscall_handler = CheatableSyscallHandler {
        syscall_handler,
        cheatnet_state: &mut CheatnetState::default(),
    };

    let mut cheatcodes_hint_processor = CheatcodesSyscallHandler {
        cheatable_syscall_handler,
        contracts,
        hints: &string_to_hint,
        run_resources: RunResources::default(),
        environment_variables,
    };

    match runner.run_function(
        runner.find_function(case.name.as_str())?,
        &mut cheatcodes_hint_processor,
        hints_dict,
        instructions,
        builtins,
    ) {
        Ok(result) => Ok(TestCaseSummary::from_run_result(
            result,
            case,
            args,
            fork_params,
        )),

        // CairoRunError comes from VirtualMachineError which may come from HintException that originates in the cheatcode processor
        Err(RunnerError::CairoRunError(error)) => Ok(TestCaseSummary::Failed {
            name: case.name.clone(),
            run_result: None,
            msg: Some(format!(
                "\n    {}\n",
                error.to_string().replace(" Custom Hint Error: ", "\n    ")
            )),
            arguments: args,
            fork_params,
        }),

        Err(err) => Err(err.into()),
    }
}

fn get_fork_state_reader(
    workspace_root: &Utf8PathBuf,
    fork_params: Option<&(Url, BlockId)>,
) -> Option<ForkStateReader> {
    fork_params.map(|(url, block_id)| {
        ForkStateReader::new(
            url,
            *block_id,
            Some(workspace_root.join(CACHE_FILE_NAME).as_ref()),
        )
    })
}

pub(crate) fn extract_fork_params(
    fork_targets: &[ForkTarget],
    fork_config: &Option<ForkConfig>,
) -> Result<Option<(Url, BlockId)>> {
    let result = match fork_config {
        Some(ForkConfig::Id(name)) => {
            let fork_target = fork_targets
                .iter()
                .find(|fork| fork.name == *name)
                .ok_or_else(|| {
                    anyhow!(
                        "The fork name used in `#[fork({name})]` attribute is not present in the Scarb.toml. \
                        Make sure to include a fork configuration with a matching name in your Scarb.toml."
                    )
                })?;
            Some((fork_target.url.clone(), fork_target.block_id))
        }
        Some(ForkConfig::Params(url, block_id)) => Some((url.clone(), *block_id)),
        None => None,
    };

    Ok(result)
}
