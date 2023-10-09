use crate::state::CheatnetState;
use blockifier::execution::common_hints::HintExecutionResult;
use blockifier::execution::deprecated_syscalls::hint_processor::{
    DeprecatedSyscallExecutionError, DeprecatedSyscallHintProcessor,
};
use blockifier::execution::deprecated_syscalls::{
    CallContractRequest, DeprecatedSyscallResult, DeprecatedSyscallSelector, EmptyRequest,
    GetBlockNumberResponse, GetBlockTimestampResponse, GetContractAddressResponse, SyscallRequest,
    SyscallResponse, WriteResponseResult,
};
use blockifier::execution::execution_utils::{write_maybe_relocatable, ReadOnlySegment};
use blockifier::execution::hint_code;

use crate::execution::calls::cairo0_calls::execute_library_call;
use crate::execution::syscalls::stark_felt_from_ptr_immutable;
use cairo_felt::Felt252;
use cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData;
use cairo_vm::hint_processor::hint_processor_definition::{HintProcessorLogic, HintReference};
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::types::relocatable::Relocatable;
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cairo_vm::vm::vm_core::VirtualMachine;
use num_traits::ToPrimitive;
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::core::ContractAddress;
use std::any::Any;
use std::collections::HashMap;

#[derive(Debug)]
// crates/blockifier/src/execution/syscalls/mod.rs:127 (SingleSegmentResponse)
// It is created here because fields in the original structure are private
// so we cannot create it in call_contract_syscall
pub struct SingleSegmentResponse {
    pub(crate) segment: ReadOnlySegment,
}
// crates/blockifier/src/execution/syscalls/mod.rs:131 (SyscallResponse for SingleSegmentResponse)
impl SyscallResponse for SingleSegmentResponse {
    fn write(self, vm: &mut VirtualMachine, ptr: &mut Relocatable) -> WriteResponseResult {
        write_maybe_relocatable(vm, ptr, self.segment.length)?;
        write_maybe_relocatable(vm, ptr, self.segment.start_ptr)?;
        Ok(())
    }
}

pub struct Cairo0CheatableSyscallHandler<'a> {
    pub syscall_handler: DeprecatedSyscallHintProcessor<'a>,
    pub cheatnet_state: &'a mut CheatnetState,
}

impl ResourceTracker for Cairo0CheatableSyscallHandler<'_> {
    fn consumed(&self) -> bool {
        self.syscall_handler.context.vm_run_resources.consumed()
    }

    fn consume_step(&mut self) {
        self.syscall_handler.context.vm_run_resources.consume_step();
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.syscall_handler.context.vm_run_resources.get_n_steps()
    }

    fn run_resources(&self) -> &RunResources {
        self.syscall_handler
            .context
            .vm_run_resources
            .run_resources()
    }
}

impl HintProcessorLogic for Cairo0CheatableSyscallHandler<'_> {
    fn execute_hint(
        &mut self,
        vm: &mut VirtualMachine,
        exec_scopes: &mut ExecutionScopes,
        hint_data: &Box<dyn Any>,
        constants: &HashMap<String, Felt252>,
    ) -> HintExecutionResult {
        let hint = hint_data
            .downcast_ref::<HintProcessorData>()
            .ok_or(HintError::WrongHintData)?;
        if hint_code::SYSCALL_HINTS.contains(hint.code.as_str()) {
            return self.execute_next_syscall_cheated(vm, &hint.ids_data, &hint.ap_tracking);
        }

        self.syscall_handler
            .execute_hint(vm, exec_scopes, hint_data, constants)
    }
}

impl<'a> Cairo0CheatableSyscallHandler<'a> {
    /// Infers and executes the next syscall.
    /// Must comply with the API of a hint function, as defined by the `HintProcessor`.
    pub fn execute_next_syscall_cheated(
        &mut self,
        vm: &mut VirtualMachine,
        ids_data: &HashMap<String, HintReference>,
        ap_tracking: &ApTracking,
    ) -> HintExecutionResult {
        // We peak into the selector without incrementing the pointer as it is done later
        let syscall_selector_pointer = self.syscall_handler.syscall_ptr;
        let selector = DeprecatedSyscallSelector::try_from(stark_felt_from_ptr_immutable(
            vm,
            &syscall_selector_pointer,
        )?)?;
        let contract_address = self.syscall_handler.storage_address;

        if DeprecatedSyscallSelector::GetCallerAddress == selector
            && self.cheatnet_state.address_is_cheated(&contract_address)
        {
            self.syscall_handler.syscall_ptr += 1;

            let response =
                get_caller_address(&EmptyRequest {}, vm, self, contract_address).unwrap();

            response.write(vm, &mut self.syscall_handler.syscall_ptr)?;

            return Ok(());
        } else if DeprecatedSyscallSelector::GetBlockNumber == selector {
            self.syscall_handler.syscall_ptr += 1;

            let response = get_block_number(&EmptyRequest {}, vm, self, contract_address).unwrap();

            response.write(vm, &mut self.syscall_handler.syscall_ptr)?;

            return Ok(());
        } else if DeprecatedSyscallSelector::GetBlockTimestamp == selector {
            self.syscall_handler.syscall_ptr += 1;

            let response =
                get_block_timestamp(&EmptyRequest {}, vm, self, contract_address).unwrap();

            response.write(vm, &mut self.syscall_handler.syscall_ptr)?;

            return Ok(());
        } else if DeprecatedSyscallSelector::DelegateCall == selector {
            self.syscall_handler.syscall_ptr += 1;
            return self.execute_syscall(vm, delegate_call);
        };

        self.syscall_handler
            .execute_next_syscall(vm, ids_data, ap_tracking)
    }

    fn execute_syscall<Request, Response, ExecuteCallback>(
        &mut self,
        vm: &mut VirtualMachine,
        execute_callback: ExecuteCallback,
    ) -> HintExecutionResult
    where
        Request: SyscallRequest,
        Response: SyscallResponse,
        ExecuteCallback: FnOnce(
            Request,
            &mut VirtualMachine,
            &mut Cairo0CheatableSyscallHandler<'_>,
        ) -> DeprecatedSyscallResult<Response>,
    {
        let request = Request::read(vm, &mut self.syscall_handler.syscall_ptr)?;

        let response = execute_callback(request, vm, self)?;
        response.write(vm, &mut self.syscall_handler.syscall_ptr)?;

        Ok(())
    }

    pub fn verify_syscall_ptr(&self, actual_ptr: Relocatable) -> DeprecatedSyscallResult<()> {
        if actual_ptr != self.syscall_handler.syscall_ptr {
            return Err(DeprecatedSyscallExecutionError::BadSyscallPointer {
                expected_ptr: self.syscall_handler.syscall_ptr,
                actual_ptr,
            });
        }

        Ok(())
    }
}

pub fn delegate_call(
    request: CallContractRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut Cairo0CheatableSyscallHandler<'_>,
) -> DeprecatedSyscallResult<SingleSegmentResponse> {
    let call_to_external = true;
    let storage_address = request.contract_address;
    let class_hash = syscall_handler
        .syscall_handler
        .state
        .get_class_hash_at(storage_address)?;
    let retdata_segment = execute_library_call(
        syscall_handler,
        vm,
        class_hash,
        Some(storage_address),
        call_to_external,
        request.function_selector,
        request.calldata,
    )?;

    Ok(SingleSegmentResponse {
        segment: retdata_segment,
    })
}

pub fn get_caller_address(
    _request: &EmptyRequest,
    _vm: &mut VirtualMachine,
    syscall_handler: &mut Cairo0CheatableSyscallHandler<'_>,
    contract_address: ContractAddress,
) -> DeprecatedSyscallResult<GetContractAddressResponse> {
    Ok(GetContractAddressResponse {
        address: *syscall_handler
            .cheatnet_state
            .pranked_contracts
            .get(&contract_address)
            .unwrap(),
    })
}

pub fn get_block_number(
    _request: &EmptyRequest,
    _vm: &mut VirtualMachine,
    syscall_handler: &mut Cairo0CheatableSyscallHandler<'_>,
    contract_address: ContractAddress,
) -> DeprecatedSyscallResult<GetBlockNumberResponse> {
    Ok(GetBlockNumberResponse {
        block_number: BlockNumber(
            syscall_handler
                .cheatnet_state
                .rolled_contracts
                .get(&contract_address)
                .unwrap()
                .to_u64()
                .unwrap(),
        ),
    })
}

pub fn get_block_timestamp(
    _request: &EmptyRequest,
    _vm: &mut VirtualMachine,
    syscall_handler: &mut Cairo0CheatableSyscallHandler<'_>,
    contract_address: ContractAddress,
) -> DeprecatedSyscallResult<GetBlockTimestampResponse> {
    Ok(GetBlockTimestampResponse {
        block_timestamp: BlockTimestamp(
            syscall_handler
                .cheatnet_state
                .warped_contracts
                .get(&contract_address)
                .unwrap()
                .to_u64()
                .unwrap(),
        ),
    })
}
