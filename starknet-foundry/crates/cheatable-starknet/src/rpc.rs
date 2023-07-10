use anyhow::Result;
use std::sync::Arc;

use blockifier::{state::cached_state::CachedState, execution::entry_point::{CallEntryPoint, CallType, ExecutionResources, EntryPointExecutionContext}};
use cairo_felt_blockifier::Felt252;
use starknet_api::{core::{ContractAddress, PatriciaKey, EntryPointSelector}, hash::{StarkFelt, StarkHash}, patricia_key, transaction::Calldata, state::EntryPointType};

use crate::{state::DictStateReader, constants::{TEST_ACCOUNT_CONTRACT_ADDRESS, build_transaction_context, build_block_context}};


// This can mutate state, the name of the syscall is not very good
fn call_contract(
    contract_address: &Felt252,
    entry_point_selector: &Felt252,
    calldata: &[Felt252],
    blockifier_state: &mut CachedState<DictStateReader>,
) -> Result<Vec<Felt252>> {
    let contract_address = ContractAddress(PatriciaKey::try_from(StarkFelt::new(
        contract_address.to_be_bytes(),
    )?)?);
    let entry_point_selector =
        EntryPointSelector(StarkHash::new(entry_point_selector.to_be_bytes())?);
    let account_address = ContractAddress(patricia_key!(TEST_ACCOUNT_CONTRACT_ADDRESS));
    let calldata = Calldata(Arc::new(
        calldata
            .iter()
            .map(|data| StarkFelt::new(data.to_be_bytes()))
            .collect::<Result<Vec<_>, _>>()?,
    ));
    let entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(contract_address),
        entry_point_type: EntryPointType::External,
        entry_point_selector,
        calldata,
        storage_address: contract_address,
        caller_address: account_address,
        call_type: CallType::Call,
        initial_gas: u64::MAX,
    };

    let mut resources = ExecutionResources::default();
    let account_context = build_transaction_context();
    let block_context = build_block_context();

    let mut context = EntryPointExecutionContext::new(
        block_context.clone(),
        account_context,
        block_context.invoke_tx_max_n_steps,
    );

    let call_info = entry_point
        .execute(blockifier_state, &mut resources, &mut context)
        .unwrap();

    let raw_return_data = &call_info.execution.retdata.0;
    assert!(!call_info.execution.failed);

    let return_data = raw_return_data
        .iter()
        .map(|data| Felt252::from_bytes_be(data.bytes()))
        .collect();

    Ok(return_data)
}