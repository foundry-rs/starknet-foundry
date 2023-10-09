use crate::execution::syscalls::lib::stark_felt_from_ptr_immutable;
use crate::state::CheatnetState;
use blockifier::abi::constants;
use blockifier::execution::common_hints::HintExecutionResult;
use blockifier::execution::deprecated_syscalls::hint_processor::{
    DeprecatedSyscallExecutionError, DeprecatedSyscallHintProcessor,
};
use blockifier::execution::deprecated_syscalls::{
    CallContractRequest, DeprecatedSyscallResult, DeprecatedSyscallSelector, EmptyRequest,
    GetContractAddressResponse, SyscallRequest, SyscallResponse, WriteResponseResult,
};
use blockifier::execution::entry_point::{CallEntryPoint, CallType};
use blockifier::execution::execution_utils::{
    stark_felt_to_felt, write_maybe_relocatable, ReadOnlySegment,
};
use blockifier::execution::hint_code;

use crate::execution::entry_point::execute_call_entry_point;
use cairo_felt::Felt252;
use cairo_vm::hint_processor::builtin_hint_processor::builtin_hint_processor_definition::HintProcessorData;
use cairo_vm::hint_processor::hint_processor_definition::{HintProcessorLogic, HintReference};
use cairo_vm::serde::deserialize_program::ApTracking;
use cairo_vm::types::exec_scope::ExecutionScopes;
use cairo_vm::types::relocatable::{MaybeRelocatable, Relocatable};
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::runners::cairo_runner::{ResourceTracker, RunResources};
use cairo_vm::vm::vm_core::VirtualMachine;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector};
use starknet_api::deprecated_contract_class::EntryPointType;
use starknet_api::transaction::Calldata;
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
    pub fn verify_syscall_ptr(&self, actual_ptr: Relocatable) -> DeprecatedSyscallResult<()> {
        if actual_ptr != self.syscall_handler.syscall_ptr {
            return Err(DeprecatedSyscallExecutionError::BadSyscallPointer {
                expected_ptr: self.syscall_handler.syscall_ptr,
                actual_ptr,
            });
        }

        Ok(())
    }

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

pub fn execute_inner_call(
    call: &mut CallEntryPoint,
    vm: &mut VirtualMachine,
    syscall_handler: &mut Cairo0CheatableSyscallHandler<'_>,
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
        .map(|&x| MaybeRelocatable::from(stark_felt_to_felt(x)))
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

pub fn execute_library_call(
    syscall_handler: &mut Cairo0CheatableSyscallHandler<'_>,
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
