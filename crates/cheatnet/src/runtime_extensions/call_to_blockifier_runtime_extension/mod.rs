use std::marker::PhantomData;

use crate::constants::TEST_ADDRESS;
use blockifier::execution::entry_point::{CallEntryPoint, CallType};
use blockifier::execution::execution_utils::felt_from_ptr;
use blockifier::execution::syscalls::{
    CallContractRequest, LibraryCallRequest, SyscallRequestWrapper,
};
use blockifier::execution::{
    deprecated_syscalls::DeprecatedSyscallSelector,
    execution_utils::ReadOnlySegment,
    syscalls::{
        hint_processor::SyscallHintProcessor, SyscallRequest, SyscallResponse,
        SyscallResponseWrapper,
    },
};
use cairo_vm::types::relocatable::MaybeRelocatable;
use cairo_vm::vm::{errors::hint_errors::HintError, vm_core::VirtualMachine};
use runtime::{ExtendedRuntime, ExtensionLogic, SyscallHandlingResult, SyscallPtrAccess};
use starknet_api::contract_class::EntryPointType;
use starknet_api::core::ContractAddress;

use crate::state::CheatnetState;

use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    call_entry_point, AddressOrClassHash,
};
use crate::runtime_extensions::call_to_blockifier_runtime_extension::{
    execution::cheated_syscalls::SingleSegmentResponse,
    rpc::{CallFailure, CallResult},
};

use super::cheatable_starknet_runtime_extension::CheatableStarknetRuntime;
use conversions::string::TryFromHexStr;

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
        selector: DeprecatedSyscallSelector,
        vm: &mut VirtualMachine,
        extended_runtime: &mut Self::Runtime,
    ) -> Result<SyscallHandlingResult, HintError> {
        match selector {
            // We execute contract calls and library calls with modified blockifier
            // This is redirected to drop ForgeRuntimeExtension
            // and to enable handling call errors with safe dispatchers in the test code
            // since call errors cannot be handled on real starknet
            // https://docs.starknet.io/architecture-and-concepts/smart-contracts/system-calls-cairo1/#call_contract
            DeprecatedSyscallSelector::CallContract => {
                execute_syscall::<CallContractRequest>(vm, extended_runtime)?;

                extended_runtime
                    .extended_runtime
                    .hint_handler
                    .increment_syscall_count_by(&selector, 1);

                Ok(SyscallHandlingResult::Handled)
            }
            DeprecatedSyscallSelector::LibraryCall => {
                execute_syscall::<LibraryCallRequest>(vm, extended_runtime)?;

                extended_runtime
                    .extended_runtime
                    .hint_handler
                    .increment_syscall_count_by(&selector, 1);

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
    ) -> CallResult;
}

impl ExecuteCall for CallContractRequest {
    fn execute_call(
        self: CallContractRequest,
        syscall_handler: &mut SyscallHintProcessor,
        cheatnet_state: &mut CheatnetState,
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
            initial_gas: i64::MAX as u64,
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
            initial_gas: u64::MAX,
        };

        call_entry_point(
            syscall_handler,
            cheatnet_state,
            entry_point,
            &AddressOrClassHash::ClassHash(class_hash),
        )
    }
}

fn execute_syscall<Request: ExecuteCall + SyscallRequest>(
    vm: &mut VirtualMachine,
    cheatable_starknet_runtime: &mut CheatableStarknetRuntime,
) -> Result<(), HintError> {
    let _selector = felt_from_ptr(vm, cheatable_starknet_runtime.get_mut_syscall_ptr())?;

    let SyscallRequestWrapper {
        gas_counter,
        request,
    } = SyscallRequestWrapper::<Request>::read(
        vm,
        cheatable_starknet_runtime.get_mut_syscall_ptr(),
    )?;

    let cheatnet_state = &mut *cheatable_starknet_runtime.extension.cheatnet_state;
    let syscall_handler = &mut cheatable_starknet_runtime.extended_runtime.hint_handler;

    let call_result = request.execute_call(syscall_handler, cheatnet_state);
    write_call_response(syscall_handler, vm, gas_counter, call_result)?;
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
                error_data: panic_data,
            },
            CallFailure::Error { msg } => {
                return Err(HintError::CustomHint(Box::from(msg.to_string())))
            }
        },
    };

    response_wrapper.write(vm, &mut syscall_handler.syscall_ptr)?;

    Ok(())
}
