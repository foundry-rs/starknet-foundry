use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::entry_point::{
    ExecuteCallEntryPointExtraOptions, execute_call_entry_point,
};
use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::execution_utils::clear_events_and_messages_from_reverted_call;
use crate::runtime_extensions::native::native_syscall_handler::BaseSyscallResult;
use crate::state::CheatnetState;
use blockifier::execution::entry_point::CallEntryPoint;
use blockifier::execution::syscalls::hint_processor::{
    ENTRYPOINT_FAILED_ERROR, SyscallExecutionError,
};
use blockifier::execution::syscalls::syscall_base::SyscallHandlerBase;
use starknet_types_core::felt::Felt;

// Based on https://github.com/software-mansion-labs/sequencer/blob/57447e3e8897d4e7ce7f3ec8d23af58d5b6bf1a7/crates/blockifier/src/execution/syscalls/syscall_base.rs#L435
pub fn execute_inner_call(
    // region: Modified blockifier code
    syscall_handler_base: &mut SyscallHandlerBase,
    cheatnet_state: &mut CheatnetState,
    call: &mut CallEntryPoint,
    remaining_gas: &mut u64,
) -> BaseSyscallResult<Vec<Felt>> {
    // endregion
    let revert_idx = syscall_handler_base.context.revert_infos.0.len();

    // region: Modified blockifier code
    let call_info = execute_call_entry_point(
        call,
        syscall_handler_base.state,
        cheatnet_state,
        syscall_handler_base.context,
        remaining_gas,
        &ExecuteCallEntryPointExtraOptions {
            trace_data_handled_by_revert_call: false,
        },
    )?;
    // endregion

    let mut raw_retdata = call_info.execution.retdata.0.clone();
    let failed = call_info.execution.failed;
    syscall_handler_base.inner_calls.push(call_info);
    if failed {
        syscall_handler_base
            .context
            .revert(revert_idx, syscall_handler_base.state)?;

        // Delete events and l2_to_l1_messages from the reverted call.
        let reverted_call = syscall_handler_base.inner_calls.last_mut().unwrap();
        clear_events_and_messages_from_reverted_call(reverted_call);

        raw_retdata
            .push(Felt::from_hex(ENTRYPOINT_FAILED_ERROR).map_err(SyscallExecutionError::from)?);
        return Err(SyscallExecutionError::Revert {
            error_data: raw_retdata,
        });
    }

    Ok(raw_retdata)
}
