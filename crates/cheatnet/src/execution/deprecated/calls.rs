use crate::execution::deprecated::syscalls::CheatableSyscallHandler;
use crate::execution::entry_point::execute_call_entry_point;
use blockifier::abi::constants;
use blockifier::execution::deprecated_syscalls::DeprecatedSyscallResult;
use blockifier::execution::entry_point::{CallEntryPoint, CallType};
use blockifier::execution::execution_utils::ReadOnlySegment;
use cairo_vm::types::relocatable::MaybeRelocatable;
use cairo_vm::vm::vm_core::VirtualMachine;
use conversions::StarknetConversions;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::transaction::Calldata;

// blockifier/src/execution/deprecated_syscalls/hint_processor.rs:393 (execute_inner_call)
pub fn execute_inner_call(
    call: &mut CallEntryPoint,
    vm: &mut VirtualMachine,
    syscall_handler: &mut CheatableSyscallHandler<'_>,
) -> DeprecatedSyscallResult<ReadOnlySegment> {
    // region: Modified blockifier code
    let call_info = execute_call_entry_point(
        call,
        syscall_handler.syscall_handler.state,
        syscall_handler.cheatnet_state,
        syscall_handler.syscall_handler.resources,
        syscall_handler.syscall_handler.context,
    )?;
    // endregion

    let retdata = &call_info.execution.retdata.0;
    let retdata: Vec<MaybeRelocatable> = retdata
        .iter()
        .map(|&x| MaybeRelocatable::from(x.to_felt252()))
        .collect();
    let retdata_segment_start_ptr = syscall_handler
        .syscall_handler
        .read_only_segments
        .allocate(vm, &retdata)?;

    syscall_handler.syscall_handler.inner_calls.push(call_info);
    Ok(ReadOnlySegment {
        start_ptr: retdata_segment_start_ptr,
        length: retdata.len(),
    })
}

// blockifier/src/execution/deprecated_syscalls/hint_processor.rs:409 (execute_library_call)
pub fn execute_library_call(
    syscall_handler: &mut CheatableSyscallHandler<'_>,
    vm: &mut VirtualMachine,
    class_hash: ClassHash,
    code_address: Option<ContractAddress>,
    call_to_external: bool,
    entry_point_selector: EntryPointSelector,
    calldata: Calldata,
) -> DeprecatedSyscallResult<ReadOnlySegment> {
    let entry_point_type = if call_to_external {
        EntryPointType::External
    } else {
        EntryPointType::L1Handler
    };
    let mut entry_point = CallEntryPoint {
        class_hash: Some(class_hash),
        code_address,
        entry_point_type,
        entry_point_selector,
        calldata,
        // The call context remains the same in a library call.
        storage_address: syscall_handler.syscall_handler.storage_address,
        caller_address: syscall_handler.syscall_handler.caller_address,
        call_type: CallType::Delegate,
        initial_gas: constants::INITIAL_GAS_COST,
    };

    execute_inner_call(&mut entry_point, vm, syscall_handler)
}
