use crate::runtime_extensions::call_to_blockifier_runtime_extension::CheatnetState;
use blockifier::execution::syscalls::hint_processor::ENTRYPOINT_FAILED_ERROR;
use blockifier::{
    execution::execution_utils::update_remaining_gas,
    execution::{
        entry_point::{CallEntryPoint, CallType},
        execution_utils::ReadOnlySegment,
        syscalls::{
            hint_processor::{create_retdata_segment, SyscallExecutionError, SyscallHintProcessor},
            syscall_base::SyscallResult,
        },
    },
};
use cairo_vm::vm::vm_core::VirtualMachine;
use starknet_api::{
    contract_class::EntryPointType,
    core::{ClassHash, EntryPointSelector},
    transaction::fields::Calldata,
};
use starknet_types_core::felt::Felt;

use super::entry_point::execute_call_entry_point;

// blockifier/src/execution/syscalls/hint_processor.rs:541 (execute_inner_call)
pub fn execute_inner_call(
    call: &mut CallEntryPoint,
    vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    remaining_gas: &mut u64,
) -> SyscallResult<ReadOnlySegment> {
    // region: Modified blockifier code
    let call_info = execute_call_entry_point(
        call,
        syscall_handler.base.state,
        cheatnet_state,
        // syscall_handler.resources,
        syscall_handler.base.context,
    )?;
    // endregion

    let mut raw_retdata = call_info.execution.retdata.0.clone();

    if call_info.execution.failed {
        raw_retdata
            .push(Felt::from_hex(ENTRYPOINT_FAILED_ERROR).map_err(SyscallExecutionError::from)?);
        return Err(SyscallExecutionError::Revert {
            error_data: raw_retdata,
        });
    }

    let retdata_segment = create_retdata_segment(vm, syscall_handler, &raw_retdata)?;
    update_remaining_gas(remaining_gas, &call_info);

    syscall_handler.base.inner_calls.push(call_info);

    Ok(retdata_segment)
}

// blockifier/src/execution/syscalls/hint_processor.rs:577 (execute_library_call)
#[allow(clippy::too_many_arguments)]
pub fn execute_library_call(
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    vm: &mut VirtualMachine,
    class_hash: ClassHash,
    call_to_external: bool,
    entry_point_selector: EntryPointSelector,
    calldata: Calldata,
    remaining_gas: &mut u64,
) -> SyscallResult<ReadOnlySegment> {
    let entry_point_type = if call_to_external {
        EntryPointType::External
    } else {
        EntryPointType::L1Handler
    };
    let mut entry_point = CallEntryPoint {
        class_hash: Some(class_hash),
        code_address: None,
        entry_point_type,
        entry_point_selector,
        calldata,
        // The call context remains the same in a library call.
        storage_address: syscall_handler.storage_address(),
        caller_address: syscall_handler.caller_address(),
        call_type: CallType::Delegate,
        initial_gas: *remaining_gas,
    };

    execute_inner_call(
        &mut entry_point,
        vm,
        syscall_handler,
        cheatnet_state,
        remaining_gas,
    )
}
