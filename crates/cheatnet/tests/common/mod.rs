use assertions::ClassHashAssert;
use blockifier::execution::call_info::CallInfo;
use blockifier::execution::contract_class::TrackedResource;
use blockifier::execution::entry_point::{
    CallEntryPoint, CallType, ConstructorContext, EntryPointExecutionContext,
    EntryPointExecutionResult,
};
use blockifier::execution::execution_utils::ReadOnlySegments;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::state::state_api::State;
use cairo_lang_casm::hints::Hint;
use cairo_vm::types::relocatable::Relocatable;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::execution::cheated_syscalls;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::execution::entry_point::non_reverting_execute_call_entry_point;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    AddressOrClassHash, CallSuccess, call_entry_point,
};
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallFailure, CallResult,
};
use cheatnet::runtime_extensions::common::create_execute_calldata;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use cheatnet::state::CheatnetState;
use conversions::IntoConv;
use conversions::string::TryFromHexStr;
use foundry_ui::UI;
use runtime::starknet::constants::TEST_ADDRESS;
use runtime::starknet::context::build_context;
use scarb_api::metadata::metadata_for_dir;
use scarb_api::{
    CompilationOpts, get_contracts_artifacts_and_source_sierra_paths, target_dir_for_workspace,
};
use starknet_api::contract_class::EntryPointType;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector};
use starknet_api::transaction::fields::Calldata;
use starknet_rust::core::utils::get_selector_from_name;
use starknet_types_core::felt::Felt;
use std::collections::HashMap;
use std::sync::Arc;

pub mod assertions;
pub mod cache;
pub mod state;

fn build_syscall_hint_processor<'a>(
    call_entry_point: &CallEntryPoint,
    state: &'a mut dyn State,
    entry_point_execution_context: &'a mut EntryPointExecutionContext,
    hints: &'a HashMap<String, Hint>,
) -> SyscallHintProcessor<'a> {
    let class_hash = call_entry_point.class_hash.unwrap_or_default();
    let call_entry_point = call_entry_point.clone().into_executable(class_hash);

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
        Ok(CallSuccess { ret_data, .. }) => ret_data,
        Err(failure_type) => match failure_type {
            CallFailure::Recoverable { panic_data, .. } => panic_data,
            CallFailure::Unrecoverable { msg, .. } => panic!("Call failed with message: {msg}"),
        },
    }
}

pub fn get_contracts() -> ContractsData {
    let scarb_metadata = metadata_for_dir("tests/contracts").unwrap();
    let target_dir = target_dir_for_workspace(&scarb_metadata).join("dev");

    let package = scarb_metadata.packages.first().unwrap();

    let ui = UI::default();
    let contracts = get_contracts_artifacts_and_source_sierra_paths(
        &target_dir,
        package,
        &ui,
        CompilationOpts {
            use_test_target_contracts: false,
            #[cfg(feature = "cairo-native")]
            run_native: true,
        },
    )
    .unwrap();
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

    let mut entry_point_execution_context = build_context(
        &cheatnet_state.block_info,
        None,
        &TrackedResource::CairoSteps,
    );
    let hints = HashMap::new();

    let mut syscall_hint_processor = build_syscall_hint_processor(
        &CallEntryPoint::default(),
        state,
        &mut entry_point_execution_context,
        &hints,
    );

    let (contract_address, _) = deploy_helper(
        &mut syscall_hint_processor,
        cheatnet_state,
        &class_hash,
        None,
        calldata,
    );

    contract_address
}

pub fn deploy(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    class_hash: &ClassHash,
    calldata: &[Felt],
) -> ContractAddress {
    let mut entry_point_execution_context = build_context(
        &cheatnet_state.block_info,
        None,
        &TrackedResource::CairoSteps,
    );
    let hints = HashMap::new();

    let mut syscall_hint_processor = build_syscall_hint_processor(
        &CallEntryPoint::default(),
        state,
        &mut entry_point_execution_context,
        &hints,
    );

    let (contract_address, _) = deploy_helper(
        &mut syscall_hint_processor,
        cheatnet_state,
        class_hash,
        None,
        calldata,
    );

    contract_address
}

fn deploy_helper(
    syscall_handler: &mut SyscallHintProcessor,
    cheatnet_state: &mut CheatnetState,
    class_hash: &ClassHash,
    contract_address: Option<ContractAddress>,
    calldata: &[Felt],
) -> (ContractAddress, Vec<cairo_vm::Felt252>) {
    let contract_address = contract_address
        .unwrap_or_else(|| cheatnet_state.precalculate_address(class_hash, calldata));
    let calldata = Calldata(Arc::new(calldata.to_vec()));

    let ctor_context = ConstructorContext {
        class_hash: *class_hash,
        code_address: Some(contract_address),
        storage_address: contract_address,
        caller_address: TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap(),
    };

    let call_info = cheated_syscalls::execute_deployment(
        syscall_handler.base.state,
        cheatnet_state,
        syscall_handler.base.context,
        &ctor_context,
        calldata,
        &mut (i64::MAX as u64),
    )
    .unwrap();
    cheatnet_state.increment_deploy_salt_base();

    let retdata = call_info.execution.retdata.0.clone();
    syscall_handler.base.inner_calls.push(call_info);

    (contract_address, retdata)
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

    let mut entry_point_execution_context = build_context(
        &cheatnet_state.block_info,
        None,
        &TrackedResource::CairoSteps,
    );
    let hints = HashMap::new();

    let mut syscall_hint_processor = build_syscall_hint_processor(
        &entry_point,
        state,
        &mut entry_point_execution_context,
        &hints,
    );

    call_entry_point(
        &mut syscall_hint_processor,
        cheatnet_state,
        entry_point,
        &AddressOrClassHash::ContractAddress(*contract_address),
        &mut (i64::MAX as u64),
    )
}

pub fn library_call_contract(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    class_hash: &ClassHash,
    entry_point_selector: EntryPointSelector,
    calldata: &[Felt],
) -> CallResult {
    let calldata = create_execute_calldata(calldata);

    let entry_point = CallEntryPoint {
        class_hash: Some(*class_hash),
        code_address: None,
        entry_point_type: EntryPointType::External,
        entry_point_selector,
        calldata,
        storage_address: TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap(),
        caller_address: TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap(),
        call_type: CallType::Delegate,
        initial_gas: i64::MAX as u64,
    };

    let mut entry_point_execution_context = build_context(
        &cheatnet_state.block_info,
        None,
        &TrackedResource::CairoSteps,
    );
    let hints = HashMap::new();

    let mut syscall_hint_processor = build_syscall_hint_processor(
        &entry_point,
        state,
        &mut entry_point_execution_context,
        &hints,
    );

    call_entry_point(
        &mut syscall_hint_processor,
        cheatnet_state,
        entry_point,
        &AddressOrClassHash::ClassHash(*class_hash),
        &mut (i64::MAX as u64),
    )
}

pub fn call_contract_raw(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
    entry_point_selector: EntryPointSelector,
    calldata: &[Felt],
    tracked_resource: TrackedResource,
) -> EntryPointExecutionResult<CallInfo> {
    let calldata = create_execute_calldata(calldata);

    let mut entry_point = CallEntryPoint {
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

    let mut entry_point_execution_context =
        build_context(&cheatnet_state.block_info, None, &tracked_resource);
    let hints = HashMap::new();

    let syscall_hint_processor = build_syscall_hint_processor(
        &entry_point,
        state,
        &mut entry_point_execution_context,
        &hints,
    );

    non_reverting_execute_call_entry_point(
        &mut entry_point,
        syscall_hint_processor.base.state,
        cheatnet_state,
        syscall_hint_processor.base.context,
        &mut (i64::MAX as u64),
    )
}

#[must_use]
pub fn selector_from_name(name: &str) -> EntryPointSelector {
    let selector = get_selector_from_name(name).unwrap();
    selector.into_()
}
