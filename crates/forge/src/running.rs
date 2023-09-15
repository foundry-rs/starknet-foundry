use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use blockifier::execution::entry_point::{
    CallEntryPoint, CallType, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::execution::execution_utils::ReadOnlySegments;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::cached_state::{CachedState, GlobalContractCache};
use cairo_vm::serde::deserialize_program::HintParams;
use cairo_vm::types::relocatable::Relocatable;
use cheatnet::constants::{build_block_context, build_testing_state, build_transaction_context};
use cheatnet::CheatnetState;
use itertools::chain;

use cairo_lang_casm::hints::Hint;
use cairo_lang_casm::instructions::Instruction;
use cairo_lang_runner::casm_run::hint_to_hint_params;
use cairo_lang_runner::RunnerError;
use cairo_lang_runner::SierraCasmRunner;
use cairo_vm::vm::runners::cairo_runner::RunResources;
use camino::Utf8PathBuf;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::state::ExtendedStateReader;
use conversions::StarknetConversions;
use starknet::core::types::{BlockId, BlockTag};
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::PatriciaKey;
use starknet_api::core::{ContractAddress, EntryPointSelector};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkHash;
use starknet_api::patricia_key;
use starknet_api::transaction::Calldata;
use test_collector::{Fork, TestCase};

use crate::cheatcodes_hint_processor::CairoHintProcessor;
use crate::scarb;
use crate::scarb::StarknetContractArtifacts;
use crate::test_case_summary::TestCaseSummary;

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

#[allow(clippy::too_many_lines)]
pub(crate) fn run_from_test_case(
    runner: &SierraCasmRunner,
    case: &TestCase,
    global_forks: Option<&Vec<scarb::Fork>>,
    contracts: &HashMap<String, StarknetContractArtifacts>,
    predeployed_contracts: &Utf8PathBuf,
) -> Result<TestCaseSummary> {
    let available_gas = if let Some(available_gas) = &case.available_gas {
        Some(*available_gas)
    } else {
        Some(usize::MAX)
    };

    let func = runner.find_function(case.name.as_str())?;
    let initial_gas = runner.get_initial_available_gas(func, available_gas)?;
    let (entry_code, builtins) = runner.create_entry_code(func, &[], initial_gas)?;
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
        code_address: Some(ContractAddress::from(0_u8)),
        entry_point_type: EntryPointType::External,
        entry_point_selector,
        calldata: Calldata(Arc::new(vec![])),
        storage_address: ContractAddress(patricia_key!("0x0")),
        caller_address: ContractAddress::default(),
        call_type: CallType::Call,
        initial_gas: u64::MAX,
    };

    let mut blockifier_state = CachedState::new(
        build_testing_state(predeployed_contracts),
        GlobalContractCache::default(),
    );
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
    let mut cairo_hint_processor = CairoHintProcessor {
        blockifier_syscall_handler: syscall_handler,
        contracts,
        cheatnet_state: CheatnetState::new(ExtendedStateReader {
            dict_state_reader: build_testing_state(predeployed_contracts),
            fork_state_reader: match &case.fork_config {
                Some(Fork::Config(url, block_id)) => Some(ForkStateReader::new(url, *block_id)),
                Some(Fork::Id(name)) => {
                    let fork = global_forks.unwrap().iter().find(|fork| &fork.name == name);
                    if let Some(fork) = fork {
                        let block_id = fork
                            .block_id
                            .iter()
                            .map(|(id_type, value)| match id_type.as_str() {
                                "number" => BlockId::Number(value.parse().unwrap()),
                                "hash" => BlockId::Hash(value.to_field_element()),
                                "tag" => match value.as_str() {
                                    "Latest" => BlockId::Tag(BlockTag::Latest),
                                    "Pending" => BlockId::Tag(BlockTag::Pending),
                                    _ => unreachable!(),
                                },
                                _ => unreachable!(),
                            })
                            .collect::<Vec<_>>();
                        let block_id = block_id[0];

                        Some(ForkStateReader::new(&fork.url, block_id))
                    } else {
                        None
                    }
                }
                _ => None,
            },
        }),
        hints: &string_to_hint,
        run_resources: RunResources::default(),
    };

    match runner.run_function(
        runner.find_function(case.name.as_str())?,
        &mut cairo_hint_processor,
        hints_dict,
        instructions,
        builtins,
    ) {
        Ok(result) => Ok(TestCaseSummary::from_run_result(result, case)),

        // CairoRunError comes from VirtualMachineError which may come from HintException that originates in the cheatcode processor
        Err(RunnerError::CairoRunError(error)) => Ok(TestCaseSummary::Failed {
            name: case.name.clone(),
            run_result: None,
            msg: Some(format!(
                "\n    {}\n",
                error.to_string().replace(" Custom Hint Error: ", "\n    ")
            )),
        }),

        Err(err) => Err(err.into()),
    }
}
