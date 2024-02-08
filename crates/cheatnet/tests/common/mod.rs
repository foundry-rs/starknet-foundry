use blockifier::execution::entry_point::{
    CallEntryPoint, CallType, EntryPointExecutionContext, ExecutionResources,
};
use blockifier::execution::execution_utils::ReadOnlySegments;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::state_api::State;
use cairo_felt::Felt252;
use cairo_lang_casm::hints::Hint;
use cairo_vm::types::relocatable::Relocatable;
use camino::Utf8PathBuf;
use cheatnet::constants::TEST_ADDRESS;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    call_entry_point, AddressOrClassHash,
};
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallFailure, CallResult,
};
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::RuntimeState;
use cheatnet::runtime_extensions::common::{create_entry_point_selector, create_execute_calldata};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::{
    deploy, deploy_at,
};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::CheatcodeError;
use conversions::felt252::FromShortString;
use runtime::starknet::context::build_context;
use scarb_api::metadata::MetadataCommandExt;
use scarb_api::{get_contracts_map, ScarbCommand, StarknetContractArtifacts};
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::PatriciaKey;
use starknet_api::core::{ClassHash, ContractAddress};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkHash;
use starknet_api::patricia_key;
use std::collections::HashMap;

pub mod assertions;
pub mod cache;
pub mod state;

fn build_syscall_hint_processor<'a>(
    call_entry_point: CallEntryPoint,
    state: &'a mut dyn State,
    execution_resources: &'a mut ExecutionResources,
    entry_point_execution_context: &'a mut EntryPointExecutionContext,
    hints: &'a HashMap<String, Hint>,
) -> SyscallHintProcessor<'a> {
    SyscallHintProcessor::new(
        state,
        execution_resources,
        entry_point_execution_context,
        Relocatable {
            segment_index: 0,
            offset: 0,
        },
        call_entry_point,
        hints,
        ReadOnlySegments::default(),
    )
}
pub fn recover_data(output: CallResult) -> Vec<Felt252> {
    match output {
        CallResult::Success { ret_data, .. } => ret_data,
        CallResult::Failure(failure_type) => match failure_type {
            CallFailure::Panic { panic_data, .. } => panic_data,
            CallFailure::Error { msg, .. } => panic!("Call failed with message: {msg}"),
        },
    }
}

pub fn get_contracts() -> HashMap<String, StarknetContractArtifacts> {
    let scarb_metadata = ScarbCommand::metadata()
        .inherit_stderr()
        .manifest_path(Utf8PathBuf::from("tests/contracts/Scarb.toml"))
        .run()
        .unwrap();

    let package = scarb_metadata.packages.first().unwrap();
    get_contracts_map(&scarb_metadata, &package.id, None).unwrap()
}

pub fn deploy_contract(
    state: &mut dyn State,
    runtime_state: &mut RuntimeState,
    contract_name: &str,
    calldata: &[Felt252],
) -> ContractAddress {
    let contract = Felt252::from_short_string(contract_name).unwrap();
    let contracts = get_contracts();

    let class_hash = declare(state, &contract, &contracts).unwrap();

    let mut execution_resources = ExecutionResources::default();
    let mut entry_point_execution_context = build_context(runtime_state.cheatnet_state.block_info);
    let hints = HashMap::new();

    let mut syscall_hint_processor = build_syscall_hint_processor(
        CallEntryPoint::default(),
        state,
        &mut execution_resources,
        &mut entry_point_execution_context,
        &hints,
    );

    deploy(
        &mut syscall_hint_processor,
        runtime_state,
        &class_hash,
        calldata,
    )
    .unwrap()
}

pub fn deploy_wrapper(
    state: &mut dyn State,
    runtime_state: &mut RuntimeState,
    class_hash: &ClassHash,
    calldata: &[Felt252],
) -> Result<ContractAddress, CheatcodeError> {
    let mut execution_resources = ExecutionResources::default();
    let mut entry_point_execution_context = build_context(runtime_state.cheatnet_state.block_info);
    let hints = HashMap::new();

    let mut syscall_hint_processor = build_syscall_hint_processor(
        CallEntryPoint::default(),
        state,
        &mut execution_resources,
        &mut entry_point_execution_context,
        &hints,
    );

    deploy(
        &mut syscall_hint_processor,
        runtime_state,
        class_hash,
        calldata,
    )
}

pub fn deploy_at_wrapper(
    state: &mut dyn State,
    runtime_state: &mut RuntimeState,
    class_hash: &ClassHash,
    calldata: &[Felt252],
    contract_address: ContractAddress,
) -> Result<ContractAddress, CheatcodeError> {
    let mut execution_resources = ExecutionResources::default();
    let mut entry_point_execution_context = build_context(runtime_state.cheatnet_state.block_info);
    let hints = HashMap::new();

    let mut syscall_hint_processor = build_syscall_hint_processor(
        CallEntryPoint::default(),
        state,
        &mut execution_resources,
        &mut entry_point_execution_context,
        &hints,
    );

    deploy_at(
        &mut syscall_hint_processor,
        runtime_state,
        class_hash,
        calldata,
        contract_address,
    )
}

pub fn call_contract_getter_by_name(
    state: &mut dyn State,
    runtime_state: &mut RuntimeState,
    contract_address: &ContractAddress,
    fn_name: &str,
) -> CallResult {
    let selector = felt_selector_from_name(fn_name);
    let result = call_contract(
        state,
        runtime_state,
        contract_address,
        &selector,
        vec![].as_slice(),
    );

    result
}

// This does contract call without the transaction layer. This way `call_contract` can return data and modify state.
// `call` and `invoke` on the transactional layer use such method under the hood.
pub fn call_contract(
    state: &mut dyn State,
    runtime_state: &mut RuntimeState,
    contract_address: &ContractAddress,
    entry_point_selector: &Felt252,
    calldata: &[Felt252],
) -> CallResult {
    let entry_point_selector = create_entry_point_selector(entry_point_selector);
    let calldata = create_execute_calldata(calldata);

    let entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(*contract_address),
        entry_point_type: EntryPointType::External,
        entry_point_selector,
        calldata,
        storage_address: *contract_address,
        caller_address: ContractAddress(patricia_key!(TEST_ADDRESS)),
        call_type: CallType::Call,
        initial_gas: u64::MAX,
    };

    let mut execution_resources = ExecutionResources::default();
    let mut entry_point_execution_context = build_context(runtime_state.cheatnet_state.block_info);
    let hints = HashMap::new();

    let mut syscall_hint_processor = build_syscall_hint_processor(
        entry_point.clone(),
        state,
        &mut execution_resources,
        &mut entry_point_execution_context,
        &hints,
    );

    call_entry_point(
        &mut syscall_hint_processor,
        runtime_state,
        entry_point,
        &AddressOrClassHash::ContractAddress(*contract_address),
    )
}

#[must_use]
pub fn felt_selector_from_name(name: &str) -> Felt252 {
    let selector = get_selector_from_name(name).unwrap();
    Felt252::from_bytes_be(&selector.to_bytes_be())
}
