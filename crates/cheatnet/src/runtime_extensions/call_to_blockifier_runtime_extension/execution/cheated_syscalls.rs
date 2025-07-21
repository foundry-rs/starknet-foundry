use super::calls::{execute_inner_call, execute_library_call};
use super::execution_info::get_cheated_exec_info_ptr;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::CheatnetState;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::entry_point::execute_constructor_entry_point;
use blockifier::execution::syscalls::hint_processor::{
    SyscallExecutionError, SyscallHintProcessor,
};
use blockifier::execution::syscalls::syscall_base::SyscallResult;
use blockifier::execution::syscalls::vm_syscall_utils::{
    CallContractRequest, DeployRequest, DeployResponse, EmptyRequest, GetBlockHashRequest,
    GetBlockHashResponse, GetExecutionInfoResponse, LibraryCallRequest, StorageReadRequest,
    StorageReadResponse, StorageWriteRequest, StorageWriteResponse, SyscallResponse,
    SyscallSelector, WriteResponseResult,
};
use blockifier::execution::{call_info::CallInfo, entry_point::ConstructorContext};
use blockifier::execution::{
    execution_utils::ReadOnlySegment, syscalls::hint_processor::write_segment,
};
use blockifier::state::errors::StateError;
use blockifier::{
    execution::entry_point::{
        CallEntryPoint, CallType, EntryPointExecutionContext, EntryPointExecutionResult,
    },
    state::state_api::State,
};
use blockifier::{
    execution::execution_utils::update_remaining_gas,
    execution::syscalls::hint_processor::create_retdata_segment,
};
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::vm_core::VirtualMachine;
use conversions::string::TryFromHexStr;
use runtime::starknet::constants::TEST_ADDRESS;
use starknet_api::core::calculate_contract_address;
use starknet_api::{
    contract_class::EntryPointType,
    core::{ClassHash, ContractAddress},
    transaction::fields::Calldata,
};

#[expect(clippy::result_large_err)]
pub fn get_execution_info_syscall(
    _request: EmptyRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    _remaining_gas: &mut u64,
) -> SyscallResult<GetExecutionInfoResponse> {
    let execution_info_ptr = syscall_handler.get_or_allocate_execution_info_segment(vm)?;

    let cheated_data = cheatnet_state.get_cheated_data(syscall_handler.storage_address());

    let ptr_cheated_exec_info = get_cheated_exec_info_ptr(vm, execution_info_ptr, &cheated_data);

    Ok(GetExecutionInfoResponse {
        execution_info_ptr: ptr_cheated_exec_info,
    })
}

// blockifier/src/execution/syscalls/mod.rs:222 (deploy_syscall)
#[expect(clippy::result_large_err)]
pub fn deploy_syscall(
    request: DeployRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    remaining_gas: &mut u64,
) -> SyscallResult<DeployResponse> {
    // Increment the Deploy syscall's linear cost counter by the number of elements in the
    // constructor calldata
    syscall_handler.increment_linear_factor_by(
        &SyscallSelector::Deploy,
        request.constructor_calldata.0.len(),
    );

    // region: Modified blockifier code
    let deployer_address = syscall_handler.storage_address();
    // endregion
    let deployer_address_for_calculation = if request.deploy_from_zero {
        ContractAddress::default()
    } else {
        deployer_address
    };

    let deployed_contract_address = calculate_contract_address(
        request.contract_address_salt,
        request.class_hash,
        &request.constructor_calldata,
        deployer_address_for_calculation,
    )?;

    let ctor_context = ConstructorContext {
        class_hash: request.class_hash,
        code_address: Some(deployed_contract_address),
        storage_address: deployed_contract_address,
        caller_address: deployer_address,
    };
    let call_info = execute_deployment(
        syscall_handler.base.state,
        cheatnet_state,
        syscall_handler.base.context,
        &ctor_context,
        request.constructor_calldata,
        *remaining_gas,
    )?;

    let constructor_retdata =
        create_retdata_segment(vm, syscall_handler, &call_info.execution.retdata.0)?;
    update_remaining_gas(remaining_gas, &call_info);

    syscall_handler.base.inner_calls.push(call_info);

    Ok(DeployResponse {
        contract_address: deployed_contract_address,
        constructor_retdata,
    })
}

// blockifier/src/execution/execution_utils.rs:217 (execute_deployment)
#[expect(clippy::result_large_err)]
pub fn execute_deployment(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    context: &mut EntryPointExecutionContext,
    ctor_context: &ConstructorContext,
    constructor_calldata: Calldata,
    remaining_gas: u64,
) -> EntryPointExecutionResult<CallInfo> {
    // Address allocation in the state is done before calling the constructor, so that it is
    // visible from it.
    let deployed_contract_address = ctor_context.storage_address;
    let current_class_hash = state.get_class_hash_at(deployed_contract_address)?;
    if current_class_hash != ClassHash::default() {
        return Err(StateError::UnavailableContractAddress(deployed_contract_address).into());
    }

    state.set_class_hash_at(deployed_contract_address, ctor_context.class_hash)?;

    let call_info = execute_constructor_entry_point(
        state,
        cheatnet_state,
        context,
        ctor_context,
        constructor_calldata,
        remaining_gas,
    )?;

    Ok(call_info)
}

// blockifier/src/execution/syscalls/mod.rs:407 (library_call)
#[expect(clippy::result_large_err)]
pub fn library_call_syscall(
    request: LibraryCallRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    remaining_gas: &mut u64,
) -> SyscallResult<SingleSegmentResponse> {
    let call_to_external = true;
    let retdata_segment = execute_library_call(
        syscall_handler,
        cheatnet_state,
        vm,
        request.class_hash,
        call_to_external,
        request.function_selector,
        request.calldata,
        remaining_gas,
    )
    .map_err(|error| match error {
        SyscallExecutionError::Revert { .. } => error,
        _ => error.as_lib_call_execution_error(
            request.class_hash,
            syscall_handler.storage_address(),
            request.function_selector,
        ),
    })?;

    Ok(SingleSegmentResponse {
        segment: retdata_segment,
    })
}

// blockifier/src/execution/syscalls/mod.rs:157 (call_contract)
#[expect(clippy::result_large_err)]
pub fn call_contract_syscall(
    request: CallContractRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    remaining_gas: &mut u64,
) -> SyscallResult<SingleSegmentResponse> {
    let storage_address = request.contract_address;
    let class_hash = syscall_handler
        .base
        .state
        .get_class_hash_at(storage_address)?;
    let selector = request.function_selector;

    let mut entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(storage_address),
        entry_point_type: EntryPointType::External,
        entry_point_selector: selector,
        calldata: request.calldata,
        storage_address,
        caller_address: syscall_handler.storage_address(),
        call_type: CallType::Call,
        initial_gas: *remaining_gas,
    };
    let retdata_segment = execute_inner_call(
        &mut entry_point,
        vm,
        syscall_handler,
        cheatnet_state,
        remaining_gas,
    )
    .map_err(|error| match error {
        SyscallExecutionError::Revert { .. } => error,
        _ => error.as_call_contract_execution_error(class_hash, storage_address, selector),
    })?;

    // region: Modified blockifier code
    Ok(SingleSegmentResponse {
        segment: retdata_segment,
    })
    // endregion
}

#[expect(clippy::needless_pass_by_value, clippy::result_large_err)]
pub fn get_block_hash_syscall(
    request: GetBlockHashRequest,
    _vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    _remaining_gas: &mut u64,
) -> SyscallResult<GetBlockHashResponse> {
    let contract_address = syscall_handler.storage_address();
    let block_number = request.block_number.0;

    let block_hash = cheatnet_state.get_block_hash_for_contract(
        contract_address,
        block_number,
        syscall_handler,
    )?;

    Ok(GetBlockHashResponse { block_hash })
}

#[expect(clippy::needless_pass_by_value, clippy::result_large_err)]
pub fn storage_read(
    request: StorageReadRequest,
    _vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    _remaining_gas: &mut u64,
) -> SyscallResult<StorageReadResponse> {
    let original_storage_address = syscall_handler.base.call.storage_address;
    maybe_modify_storage_address(syscall_handler, cheatnet_state);

    let value = syscall_handler
        .base
        .storage_read(request.address)
        .inspect_err(|_| {
            // Restore state on error before bubbling up
            syscall_handler.base.call.storage_address = original_storage_address;
        })?;

    // Restore the original storage_address
    syscall_handler.base.call.storage_address = original_storage_address;

    Ok(StorageReadResponse { value })
}

#[expect(clippy::needless_pass_by_value, clippy::result_large_err)]
pub fn storage_write(
    request: StorageWriteRequest,
    _vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    _remaining_gas: &mut u64,
) -> SyscallResult<StorageWriteResponse> {
    let original_storage_address = syscall_handler.base.call.storage_address;
    maybe_modify_storage_address(syscall_handler, cheatnet_state);

    syscall_handler
        .base
        .storage_write(request.address, request.value)
        .inspect_err(|_| {
            // Restore state on error before bubbling up
            syscall_handler.base.call.storage_address = original_storage_address;
        })?;

    // Restore the original storage_address
    syscall_handler.base.call.storage_address = original_storage_address;

    Ok(StorageWriteResponse {})
}

// This logic is used to modify the storage address to enable using `contract_state_for_testing`
// inside `interact_with_state` closure cheatcode.
fn maybe_modify_storage_address(
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
) {
    let contract_address = syscall_handler.storage_address();
    let test_address =
        TryFromHexStr::try_from_hex_str(TEST_ADDRESS).expect("Failed to parse `TEST_ADDRESS`");

    if contract_address != test_address {
        return;
    }

    let cheated_data = cheatnet_state.get_cheated_data(contract_address);
    if let Some(actual_address) = cheated_data.contract_address {
        syscall_handler.base.call.storage_address = actual_address;
    }
}

#[derive(Debug)]
// crates/blockifier/src/execution/syscalls/mod.rs:127 (SingleSegmentResponse)
// It is created here because fields in the original structure are private
// so we cannot create it in call_contract_syscall
pub struct SingleSegmentResponse {
    pub segment: ReadOnlySegment,
}
// crates/blockifier/src/execution/syscalls/mod.rs:131 (SyscallResponse for SingleSegmentResponse)
impl SyscallResponse for SingleSegmentResponse {
    fn write(self, vm: &mut VirtualMachine, ptr: &mut Relocatable) -> WriteResponseResult {
        write_segment(vm, ptr, self.segment)
    }
}
