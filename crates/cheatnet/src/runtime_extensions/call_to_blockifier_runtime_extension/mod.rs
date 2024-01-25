use std::marker::PhantomData;

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

use crate::constants::TEST_ADDRESS;
use cairo_vm::vm::{errors::hint_errors::HintError, vm_core::VirtualMachine};
use conversions::FromConv;
use runtime::{ExtendedRuntime, ExtensionLogic, SyscallHandlingResult, SyscallPtrAccess};
use starknet_api::core::PatriciaKey;
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::hash::StarkHash;
use starknet_api::{core::ContractAddress, hash::StarkFelt, patricia_key};

use crate::state::{BlockifierState, CheatnetState};

use crate::runtime_extensions::call_to_blockifier_runtime_extension::rpc::{
    call_entry_point, AddressOrClassHash,
};
use crate::runtime_extensions::call_to_blockifier_runtime_extension::{
    execution::cheated_syscalls::SingleSegmentResponse,
    rpc::{CallFailure, CallOutput, CallResult},
};

use super::io_runtime_extension::IORuntime;

pub mod execution;
pub mod panic_data;
pub mod rpc;

pub struct CallToBlockifierExtension<'a> {
    pub lifetime: &'a PhantomData<()>,
}

pub type CallToBlockifierRuntime<'a> = ExtendedRuntime<CallToBlockifierExtension<'a>>;

impl<'a> ExtensionLogic for CallToBlockifierExtension<'a> {
    type Runtime = IORuntime<'a>;

    fn override_system_call(
        &mut self,
        selector: DeprecatedSyscallSelector,
        vm: &mut VirtualMachine,
        extended_runtime: &mut IORuntime<'a>,
    ) -> Result<SyscallHandlingResult, HintError> {
        match selector {
            // We execute contract calls and library calls with modified blockifier
            // This is redirected to drop ForgeRuntimeExtension
            // and to enable handling call errors with safe dispatchers in the test code
            // since call errors cannot be handled on real starknet
            // https://docs.starknet.io/documentation/architecture_and_concepts/Smart_Contracts/system-calls-cairo1/#call_contract
            DeprecatedSyscallSelector::CallContract => {
                execute_syscall::<CallContractRequest>(vm, extended_runtime)?;
                Ok(SyscallHandlingResult::Handled(()))
            }
            DeprecatedSyscallSelector::LibraryCall => {
                execute_syscall::<LibraryCallRequest>(vm, extended_runtime)?;
                Ok(SyscallHandlingResult::Handled(()))
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
        blockifier_state: &mut BlockifierState,
        cheatnet_state: &mut CheatnetState,
    ) -> CallOutput;
}

impl ExecuteCall for CallContractRequest {
    fn execute_call(
        self: CallContractRequest,
        blockifier_state: &mut BlockifierState,
        cheatnet_state: &mut CheatnetState,
    ) -> CallOutput {
        let contract_address = self.contract_address;

        let entry_point = CallEntryPoint {
            class_hash: None,
            code_address: Some(contract_address),
            entry_point_type: EntryPointType::External,
            entry_point_selector: self.function_selector,
            calldata: self.calldata,
            storage_address: contract_address,
            caller_address: ContractAddress(patricia_key!(TEST_ADDRESS)),
            call_type: CallType::Call,
            initial_gas: u64::MAX,
        };

        call_entry_point(
            blockifier_state,
            cheatnet_state,
            entry_point,
            &AddressOrClassHash::ContractAddress(contract_address),
        )
        .unwrap_or_else(|err| panic!("Transaction execution error: {err}"))
    }
}

impl ExecuteCall for LibraryCallRequest {
    fn execute_call(
        self: LibraryCallRequest,
        blockifier_state: &mut BlockifierState,
        cheatnet_state: &mut CheatnetState,
    ) -> CallOutput {
        let class_hash = self.class_hash;

        let entry_point = CallEntryPoint {
            class_hash: Some(class_hash),
            code_address: None,
            entry_point_type: EntryPointType::External,
            entry_point_selector: self.function_selector,
            calldata: self.calldata,
            storage_address: ContractAddress(patricia_key!(TEST_ADDRESS)),
            caller_address: ContractAddress::default(),
            call_type: CallType::Delegate,
            initial_gas: u64::MAX,
        };

        call_entry_point(
            blockifier_state,
            cheatnet_state,
            entry_point,
            &AddressOrClassHash::ClassHash(class_hash),
        )
        .unwrap_or_else(|err| panic!("Transaction execution error: {err}"))
    }
}

fn execute_syscall<Request: ExecuteCall + SyscallRequest>(
    vm: &mut VirtualMachine,
    io_runtime: &mut IORuntime,
) -> Result<(), HintError> {
    let _selector = felt_from_ptr(vm, io_runtime.get_mut_syscall_ptr())?;

    let SyscallRequestWrapper {
        gas_counter,
        request,
    } = SyscallRequestWrapper::<Request>::read(vm, io_runtime.get_mut_syscall_ptr())?;

    let cheatable_starknet_runtime = &mut io_runtime.extended_runtime;
    let cheatnet_state: &mut _ = cheatable_starknet_runtime.extension.cheatnet_state;
    let syscall_handler = &mut cheatable_starknet_runtime.extended_runtime.hint_handler;
    let mut blockifier_state = BlockifierState::from(syscall_handler.state);

    let call_result = request.execute_call(&mut blockifier_state, cheatnet_state);
    write_call_response(
        syscall_handler,
        cheatnet_state,
        vm,
        gas_counter,
        call_result,
    )?;
    Ok(())
}

fn write_call_response(
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    vm: &mut VirtualMachine,
    gas_counter: u64,
    call_output: CallOutput,
) -> Result<(), HintError> {
    let response_wrapper: SyscallResponseWrapper<SingleSegmentResponse> = match call_output.result {
        CallResult::Success { ret_data, .. } => {
            let memory_segment_start_ptr = syscall_handler
                .read_only_segments
                .allocate(vm, &ret_data.iter().map(Into::into).collect())?;

            // add execution resources used by call to all used resources
            cheatnet_state
                .used_resources
                .extend(&call_output.used_resources);

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
            CallFailure::Panic { panic_data, .. } => SyscallResponseWrapper::Failure {
                gas_counter,
                error_data: panic_data
                    .iter()
                    .map(|el| StarkFelt::from_(el.clone()))
                    .collect(),
            },
            CallFailure::Error { msg, .. } => return Err(HintError::CustomHint(Box::from(msg))),
        },
    };

    response_wrapper.write(vm, &mut syscall_handler.syscall_ptr)?;

    Ok(())
}
