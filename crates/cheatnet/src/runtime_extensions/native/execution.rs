use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::entry_point::{
    CallInfoWithExecutionData, ContractClassEntryPointExecutionResult,
};
use crate::state::CheatnetState;
use blockifier::execution::entry_point::{EntryPointExecutionContext, ExecutableCallEntryPoint};
use blockifier::execution::native::contract_class::NativeCompiledClassV1;
use blockifier::execution::native::entry_point_execution::execute_entry_point_call;
use blockifier::state::state_api::State;
use std::default::Default;

pub(crate) fn execute_entry_point_call_native(
    call: ExecutableCallEntryPoint,
    native_compiled_class_v1: NativeCompiledClassV1,
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState, // Added parameter
    context: &mut EntryPointExecutionContext,
) -> ContractClassEntryPointExecutionResult {
    // TODO error handling
    let call_info = execute_entry_point_call(call, native_compiled_class_v1, state, context)
        .expect("Native execution failed");

    Ok(CallInfoWithExecutionData {
        call_info,
        syscall_usage_vm_resources: Default::default(),
        syscall_usage_sierra_gas: Default::default(),
        vm_trace: None,
    })
}
