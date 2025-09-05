use std::marker::PhantomData;

use crate::state::CheatnetState;
use blockifier::execution::entry_point::{CallEntryPoint, CallType};
use blockifier::execution::syscalls::hint_processor::{OUT_OF_GAS_ERROR, SyscallHintProcessor};
use blockifier::execution::syscalls::syscall_executor::SyscallExecutor;
use blockifier::execution::syscalls::vm_syscall_utils::{
    CallContractRequest, LibraryCallRequest, RevertData, SingleSegmentResponse,
    SyscallExecutorBaseError, SyscallRequestWrapper, SyscallSelector,
};
use blockifier::execution::{
    execution_utils::ReadOnlySegment,
    syscalls::vm_syscall_utils::{SyscallRequest, SyscallResponse, SyscallResponseWrapper},
};
use blockifier::utils::u64_from_usize;
use cairo_vm::types::relocatable::MaybeRelocatable;
use cairo_vm::vm::{errors::hint_errors::HintError, vm_core::VirtualMachine};
use runtime::{ExtendedRuntime, ExtensionLogic, SyscallHandlingResult};
use starknet_api::contract_class::EntryPointType;
use starknet_api::core::ContractAddress;
use starknet_api::execution_resources::GasAmount;
use starknet_types_core::felt::Felt;

use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    AddressOrClassHash, call_entry_point,
};
use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    CallFailure, CallResult,
};

use super::cheatable_starknet_runtime_extension::CheatableStarknetRuntime;
use conversions::string::TryFromHexStr;
use runtime::starknet::constants::TEST_ADDRESS;

pub mod execution;
pub mod panic_data;
pub mod rpc;

pub struct CallToBlockifierExtension<'a> {
    pub lifetime: &'a PhantomData<()>,
}

pub type CallToBlockifierRuntime<'a> = ExtendedRuntime<CallToBlockifierExtension<'a>>;

impl<'a> ExtensionLogic for CallToBlockifierExtension<'a> {
    type Runtime = CheatableStarknetRuntime<'a>;

    fn override_system_call(
        &mut self,
        selector: SyscallSelector,
        vm: &mut VirtualMachine,
        extended_runtime: &mut Self::Runtime,
    ) -> Result<SyscallHandlingResult, HintError> {
        match selector {
            // We execute contract calls and library calls with modified blockifier
            // This is redirected to drop ForgeRuntimeExtension
            // and to enable executing outer calls in tests as non-revertible.
            SyscallSelector::CallContract => {
                execute_syscall::<CallContractRequest>(selector, vm, extended_runtime)?;

                Ok(SyscallHandlingResult::Handled)
            }
            SyscallSelector::LibraryCall => {
                execute_syscall::<LibraryCallRequest>(selector, vm, extended_runtime)?;

                Ok(SyscallHandlingResult::Handled)
            }
            _ => Ok(SyscallHandlingResult::Forwarded),
        }
    }
}

trait ExecuteCall
where
    Self: SyscallRequest,
{
    fn execute_call(
        self,
        syscall_handler: &mut SyscallHintProcessor,
        cheatnet_state: &mut CheatnetState,
        remaining_gas: &mut u64,
    ) -> CallResult;
}

impl ExecuteCall for CallContractRequest {
    fn execute_call(
        self: CallContractRequest,
        syscall_handler: &mut SyscallHintProcessor,
        cheatnet_state: &mut CheatnetState,
        remaining_gas: &mut u64,
    ) -> CallResult {
        let contract_address = self.contract_address;

        let entry_point = CallEntryPoint {
            class_hash: None,
            code_address: Some(contract_address),
            entry_point_type: EntryPointType::External,
            entry_point_selector: self.function_selector,
            calldata: self.calldata,
            storage_address: contract_address,
            caller_address: TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap(),
            call_type: CallType::Call,
            initial_gas: *remaining_gas,
        };

        call_entry_point(
            syscall_handler,
            cheatnet_state,
            entry_point,
            &AddressOrClassHash::ContractAddress(contract_address),
        )
    }
}

impl ExecuteCall for LibraryCallRequest {
    fn execute_call(
        self: LibraryCallRequest,
        syscall_handler: &mut SyscallHintProcessor,
        cheatnet_state: &mut CheatnetState,
        remaining_gas: &mut u64,
    ) -> CallResult {
        let class_hash = self.class_hash;

        let entry_point = CallEntryPoint {
            class_hash: Some(class_hash),
            code_address: None,
            entry_point_type: EntryPointType::External,
            entry_point_selector: self.function_selector,
            calldata: self.calldata,
            storage_address: TryFromHexStr::try_from_hex_str(TEST_ADDRESS).unwrap(),
            caller_address: ContractAddress::default(),
            call_type: CallType::Delegate,
            initial_gas: *remaining_gas,
        };

        call_entry_point(
            syscall_handler,
            cheatnet_state,
            entry_point,
            &AddressOrClassHash::ClassHash(class_hash),
        )
    }
}

// crates/blockifier/src/execution/syscalls/vm_syscall_utils.rs:677 (execute_syscall)
fn execute_syscall<Request: ExecuteCall + SyscallRequest>(
    selector: SyscallSelector,
    vm: &mut VirtualMachine,
    cheatable_starknet_runtime: &mut CheatableStarknetRuntime,
) -> Result<(), HintError> {
    // region: Modified blockifier code
    let syscall_handler = &mut cheatable_starknet_runtime.extended_runtime.hint_handler;
    let cheatnet_state = &mut *cheatable_starknet_runtime.extension.cheatnet_state;

    // Increment, since the selector was peeked into before
    syscall_handler.syscall_ptr += 1;
    syscall_handler.increment_syscall_count_by(&selector, 1);
    // endregion

    let syscall_gas_cost = syscall_handler
        .get_gas_cost_from_selector(&selector)
        .map_err(|error| SyscallExecutorBaseError::GasCost { error, selector })?;

    let SyscallRequestWrapper {
        gas_counter,
        request,
    } = SyscallRequestWrapper::<Request>::read(vm, syscall_handler.get_mut_syscall_ptr())?;

    let syscall_gas_cost =
        syscall_gas_cost.get_syscall_cost(u64_from_usize(request.get_linear_factor_length()));
    let syscall_base_cost = syscall_handler.get_syscall_base_gas_cost();

    // Sanity check for preventing underflow.
    assert!(
        syscall_gas_cost >= syscall_base_cost,
        "Syscall gas cost must be greater than base syscall gas cost"
    );

    // Refund `SYSCALL_BASE_GAS_COST` as it was pre-charged.
    // Note: It is pre-charged by the compiler: https://github.com/starkware-libs/sequencer/blob/v0.15.0-rc.2/crates/blockifier/src/blockifier_versioned_constants.rs#L1057
    let required_gas = syscall_gas_cost - syscall_base_cost;

    if gas_counter < required_gas {
        let out_of_gas_error =
            Felt::from_hex(OUT_OF_GAS_ERROR).map_err(SyscallExecutorBaseError::from)?;
        let response: SyscallResponseWrapper<SingleSegmentResponse> =
            SyscallResponseWrapper::Failure {
                gas_counter,
                revert_data: RevertData::new_normal(vec![out_of_gas_error]),
            };
        response.write(vm, syscall_handler.get_mut_syscall_ptr())?;

        return Ok(());
    }

    let mut remaining_gas = gas_counter - required_gas;

    // TODO(#3681)
    syscall_handler.update_revert_gas_with_next_remaining_gas(GasAmount(remaining_gas));

    // region: Modified blockifier code
    let call_result = request.execute_call(syscall_handler, cheatnet_state, &mut remaining_gas);
    write_call_response(syscall_handler, vm, remaining_gas, call_result)?;
    // endregion

    Ok(())
}

fn write_call_response(
    syscall_handler: &mut SyscallHintProcessor<'_>,
    vm: &mut VirtualMachine,
    gas_counter: u64,
    call_result: CallResult,
) -> Result<(), HintError> {
    let response_wrapper: SyscallResponseWrapper<SingleSegmentResponse> = match call_result {
        CallResult::Success { ret_data } => {
            let memory_segment_start_ptr = syscall_handler.read_only_segments.allocate(
                vm,
                &ret_data
                    .clone()
                    .into_iter()
                    .map(MaybeRelocatable::Int)
                    .collect::<Vec<MaybeRelocatable>>(),
            )?;

            SyscallResponseWrapper::Success {
                gas_counter,
                response: SingleSegmentResponse {
                    segment: ReadOnlySegment {
                        start_ptr: memory_segment_start_ptr,
                        length: ret_data.len(),
                    },
                },
            }
        }
        CallResult::Failure(failure_type) => match failure_type {
            CallFailure::Panic { panic_data } => SyscallResponseWrapper::Failure {
                gas_counter,
                revert_data: RevertData::new_normal(panic_data),
            },
            CallFailure::Error { msg } => {
                return Err(HintError::CustomHint(Box::from(msg.to_string())));
            }
        },
    };

    response_wrapper.write(vm, &mut syscall_handler.syscall_ptr)?;

    Ok(())
}
