use crate::abi::constants;
use self::hint_processor::{
    create_retdata_segment,
    execute_inner_call,
    execute_library_call,
    felt_to_bool,
    read_call_params,
    read_calldata,
    read_felt_array,
    write_segment,
    EmitEventError,
    SyscallExecutionError,
    SyscallHintProcessor,
    BLOCK_NUMBER_OUT_OF_RANGE_ERROR,
};

use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::entry_point::execute_constructor_entry_point;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::CheatnetState;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use blockifier::execution::syscalls::{
    DeployRequest, DeployResponse, GetBlockHashRequest, GetBlockHashResponse, LibraryCallRequest, SyscallResponse, SyscallResult
};
use blockifier::execution::{call_info::CallInfo, entry_point::ConstructorContext};
use blockifier::execution::{
    execution_utils::ReadOnlySegment,
    syscalls::{hint_processor::write_segment, WriteResponseResult},
};
use blockifier::state::errors::StateError;
use blockifier::{
    execution::syscalls::{hint_processor::create_retdata_segment, CallContractRequest},
    transaction::transaction_utils::update_remaining_gas,
};
use blockifier::{
    execution::{
        deprecated_syscalls::DeprecatedSyscallSelector,
        entry_point::{
            CallEntryPoint, CallType, EntryPointExecutionContext, EntryPointExecutionResult,
        },
        syscalls::{EmptyRequest, GetExecutionInfoResponse},
    },
    state::state_api::State,
};
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::runners::cairo_runner::ExecutionResources;
use cairo_vm::vm::vm_core::VirtualMachine;
use starknet_api::core::calculate_contract_address;
use starknet_api::{
    core::{ClassHash, ContractAddress},
    deprecated_contract_class::EntryPointType,
    transaction::Calldata,
};

use super::calls::{execute_inner_call, execute_library_call};
use super::execution_info::get_cheated_exec_info_ptr;
pub type SyscallSelector = DeprecatedSyscallSelector;

#[allow(clippy::needless_pass_by_value)]
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

pub fn get_block_hash_syscall(
    request: GetBlockHashRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    _remaining_gas: &mut u64,
) -> SyscallResult<GetBlockHashResponse> {

    // let execution_info_ptr = syscall_handler.get_or_allocate_execution_info_segment(vm)?;

    // let cheated_data = cheatnet_state.get_cheated_data(syscall_handler.storage_address());

    // let ptr_cheated_block_info = get_cheated_block_info_ptr(vm, execution_info_ptr, &cheated_data);

    let cheated_block_number = cheatnet_state.block_info.block_number;

    let requested_block_number = request.block_number;

    if cheated_block_number < constants::STORED_BLOCK_HASH_BUFFER
        || requested_block_number > cheated_block_number - constants::STORED_BLOCK_HASH_BUFFER
    {
        let out_of_range_error =
            Felt::from_hex(BLOCK_NUMBER_OUT_OF_RANGE_ERROR).map_err(SyscallExecutionError::from)?;
        return Err(SyscallExecutionError::SyscallError { error_data: vec![out_of_range_error] });
    }

    let key = StorageKey::try_from(Felt::from(requested_block_number))?;

    let block_hash_contract_address =
        ContractAddress::try_from(Felt::from(constants::BLOCK_HASH_CONTRACT_ADDRESS))?;

    //should I get the contractAddress from the cheated block number? how do I do that?

    let cheated_block_hash = BlockHash(syscall_handler.state.get_storage_at(block_hash_contract_address, key)?);

    Ok(GetBlockHashResponse { block_hash: cheated_block_hash })

}

// blockifier/src/execution/syscalls/mod.rs:222 (deploy_syscall)
pub fn deploy_syscall(
    request: DeployRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    remaining_gas: &mut u64,
) -> SyscallResult<DeployResponse> {
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
        syscall_handler.state,
        cheatnet_state,
        syscall_handler.resources,
        syscall_handler.context,
        &ctor_context,
        request.constructor_calldata,
        *remaining_gas,
    )?;

    let constructor_retdata =
        create_retdata_segment(vm, syscall_handler, &call_info.execution.retdata.0)?;
    update_remaining_gas(remaining_gas, &call_info);

    syscall_handler.inner_calls.push(call_info);

    Ok(DeployResponse {
        contract_address: deployed_contract_address,
        constructor_retdata,
    })
}

// blockifier/src/execution/execution_utils.rs:217 (execute_deployment)
pub fn execute_deployment(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    resources: &mut ExecutionResources,
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
        resources,
        context,
        ctor_context,
        constructor_calldata,
        remaining_gas,
    )?;

    Ok(call_info)
}

// blockifier/src/execution/syscalls/mod.rs:407 (library_call)
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
    )?;

    Ok(SingleSegmentResponse {
        segment: retdata_segment,
    })
}

// blockifier/src/execution/syscalls/mod.rs:157 (call_contract)
pub fn call_contract_syscall(
    request: CallContractRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    remaining_gas: &mut u64,
) -> SyscallResult<SingleSegmentResponse> {
    let storage_address = request.contract_address;
    let mut entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(storage_address),
        entry_point_type: EntryPointType::External,
        entry_point_selector: request.function_selector,
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
    )?;

    // region: Modified blockifier code
    Ok(SingleSegmentResponse {
        segment: retdata_segment,
    })
    // endregion
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
