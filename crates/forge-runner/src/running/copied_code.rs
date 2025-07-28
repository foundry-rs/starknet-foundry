// TODO(#3293) Remove this file once the copied code is upstreamed to blockifier.
//! Module containing copied code to be upstreamed to blockifier

use blockifier::execution::contract_class::TrackedResource;
use blockifier::execution::entry_point_execution::CallResult;
use blockifier::execution::errors::PostExecutionError;
use blockifier::execution::execution_utils::read_execution_retdata;
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use cairo_vm::types::relocatable::MaybeRelocatable;
use cairo_vm::vm::runners::cairo_runner::CairoRunner;
use num_traits::ToPrimitive;

#[expect(clippy::trivially_copy_pass_by_ref)]
// Reason copied: Required by `finalize_execution`
pub fn get_call_result(
    runner: &CairoRunner,
    syscall_handler: &SyscallHintProcessor<'_>,
    tracked_resource: &TrackedResource,
) -> Result<CallResult, PostExecutionError> {
    let return_result = runner.vm.get_return_values(5)?;
    // Corresponds to the Cairo 1.0 enum:
    // enum PanicResult<Array::<felt>> { Ok: Array::<felt>, Err: Array::<felt>, }.
    let [failure_flag, retdata_start, retdata_end]: &[MaybeRelocatable; 3] = (&return_result[2..])
        .try_into()
        .expect("Return values must be of size 3.");

    let failed = if *failure_flag == MaybeRelocatable::from(0) {
        false
    } else if *failure_flag == MaybeRelocatable::from(1) {
        true
    } else {
        return Err(PostExecutionError::MalformedReturnData {
            error_message: "Failure flag expected to be either 0 or 1.".to_string(),
        });
    };

    let retdata_size = retdata_end.sub(retdata_start)?;
    // TODO(spapini): Validate implicits.

    let gas = &return_result[0];
    let MaybeRelocatable::Int(gas) = gas else {
        return Err(PostExecutionError::MalformedReturnData {
            error_message: "Error extracting return data.".to_string(),
        });
    };
    let gas = gas
        .to_u64()
        .ok_or(PostExecutionError::MalformedReturnData {
            error_message: format!("Unexpected remaining gas: {gas}."),
        })?;

    if gas > syscall_handler.base.call.initial_gas {
        return Err(PostExecutionError::MalformedReturnData {
            error_message: format!("Unexpected remaining gas: {gas}."),
        });
    }

    let gas_consumed = match tracked_resource {
        // Do not count Sierra gas in CairoSteps mode.
        TrackedResource::CairoSteps => 0,
        TrackedResource::SierraGas => syscall_handler.base.call.initial_gas - gas,
    };
    Ok(CallResult {
        failed,
        retdata: read_execution_retdata(runner, retdata_size, retdata_start)?,
        gas_consumed,
    })
}
