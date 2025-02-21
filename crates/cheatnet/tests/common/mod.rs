use assertions::ClassHashAssert;
use blockifier::execution::entry_point::{CallEntryPoint, CallType, EntryPointExecutionContext};
use blockifier::execution::execution_utils::ReadOnlySegments;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::state_api::State;
use cairo_lang_casm::hints::Hint;
use cairo_vm::types::relocatable::Relocatable;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    call_entry_point, AddressOrClassHash,
};
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallFailure, CallResult,
};
use cheatnet::runtime_extensions::common::create_execute_calldata;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::{
    deploy, deploy_at,
};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::CheatcodeError;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::CheatnetState;
use conversions::string::TryFromHexStr;
use conversions::IntoConv;
use runtime::starknet::constants::TEST_ADDRESS;
use runtime::starknet::context::build_context;
use scarb_api::metadata::MetadataCommandExt;
use scarb_api::{
    get_contracts_artifacts_and_source_sierra_paths, target_dir_for_workspace, ScarbCommand,
};
use starknet::core::utils::get_selector_from_name;
use starknet_api::contract_class::EntryPointType;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector};
use starknet_types_core::felt::Felt;
use std::collections::HashMap;

pub mod assertions;
pub mod cache;
pub mod state;

fn build_syscall_hint_processor<'a>(
    call_entry_point: CallEntryPoint,
    state: &'a mut dyn State,
    entry_point_execution_context: &'a mut EntryPointExecutionContext,
    hints: &'a HashMap<String, Hint>,
) -> SyscallHintProcessor<'a> {
    SyscallHintProcessor::new(
        state,
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
pub fn recover_data(output: CallResult) -> Vec<Felt> {
    match output {
        CallResult::Success { ret_data, .. } => ret_data,
        CallResult::Failure(failure_type) => match failure_type {
            CallFailure::Panic { panic_data, .. } => panic_data,
            CallFailure::Error { msg, .. } => panic!("Call failed with message: {msg}"),
        },
    }
}

pub fn get_contracts() -> ContractsData {
    let scarb_metadata = ScarbCommand::metadata()
        .inherit_stderr()
        .manifest_path("tests/contracts/Scarb.toml")
        .run()
        .unwrap();
    let target_dir = target_dir_for_workspace(&scarb_metadata).join("dev");

    let package = scarb_metadata.packages.first().unwrap();

    let contracts =
        get_contracts_artifacts_and_source_sierra_paths(&target_dir, package, false).unwrap();
    ContractsData::try_from(contracts).unwrap()
}

pub fn deploy_contract(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    contract_name: &str,
    calldata: &[Felt],
) -> ContractAddress {
    let contracts_data = get_contracts();

    let class_hash = declare(state, contract_name, &contracts_data)
        .unwrap()
        .unwrap_success();

    let mut entry_point_execution_context = build_context(&cheatnet_state.block_info, None);
    let hints = HashMap::new();

    let mut syscall_hint_processor = build_syscall_hint_processor(
        CallEntryPoint::default(),
        state,
        &mut entry_point_execution_context,
        &hints,
    );

    let (contract_address, _retdata) = deploy(
        &mut syscall_hint_processor,
        cheatnet_state,
        &class_hash,
        calldata,
    )
    .unwrap();

    contract_address
}

pub fn deploy_wrapper(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    class_hash: &ClassHash,
    calldata: &[Felt],
) -> Result<ContractAddress, CheatcodeError> {
    let mut entry_point_execution_context = build_context(&cheatnet_state.block_info, None);
    let hints = HashMap::new();

    let mut syscall_hint_processor = build_syscall_hint_processor(
        CallEntryPoint::default(),
        state,
        &mut entry_point_execution_context,
        &hints,
    );

    let (contract_address, _retdata) = deploy(
        &mut syscall_hint_processor,
        cheatnet_state,
        class_hash,
        calldata,
    )?;

    Ok(contract_address)
}

pub fn deploy_at_wrapper(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    class_hash: &ClassHash,
    calldata: &[Felt],
    contract_address: ContractAddress,
) -> Result<ContractAddress, CheatcodeError> {
    let mut entry_point_execution_context = build_context(&cheatnet_state.block_info, None);
    let hints = HashMap::new();

    let mut syscall_hint_processor = build_syscall_hint_processor(
        CallEntryPoint::default(),
        state,
        &mut entry_point_execution_context,
        &hints,
    );

    let (contract_address, _retdata) = deploy_at(
        &mut syscall_hint_processor,
        cheatnet_state,
        class_hash,
        calldata,
        contract_address,
    )?;

    Ok(contract_address)
}

// This does contract call without the transaction layer. This way `call_contract` can return data and modify state.
// `call` and `invoke` on the transactional layer use such method under the hood.
pub fn call_contract(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
    entry_point_selector: EntryPointSelector,
    calldata: &[Felt],
) -> CallResult {
    let calldata = create_execute_calldata(calldata);

    let entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(*contract_address),
        entry_point_type: EntryPointType::External,
        entry_point_selector,
        calldata,
        storage_address: *contract_address,
        caller_address: TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap(),
        call_type: CallType::Call,
        initial_gas: i64::MAX as u64,
    };

    let mut entry_point_execution_context = build_context(&cheatnet_state.block_info, None);
    let hints = HashMap::new();

    let mut syscall_hint_processor = build_syscall_hint_processor(
        entry_point.clone(),
        state,
        &mut entry_point_execution_context,
        &hints,
    );

    call_entry_point(
        &mut syscall_hint_processor,
        cheatnet_state,
        entry_point,
        &AddressOrClassHash::ContractAddress(*contract_address),
    )
}

#[must_use]
pub fn felt_selector_from_name(name: &str) -> EntryPointSelector {
    let selector = get_selector_from_name(name).unwrap();
    selector.into_()
}
