use crate::runtime_extensions::cheatable_starknet_runtime_extension::stark_felt_from_ptr_immutable;
use crate::state::CheatnetState;
use blockifier::execution::common_hints::HintExecutionResult;
use blockifier::execution::deprecated_syscalls::hint_processor::{
    DeprecatedSyscallExecutionError, DeprecatedSyscallHintProcessor,
};
use blockifier::execution::deprecated_syscalls::{
    CallContractRequest, DeprecatedSyscallResult, DeprecatedSyscallSelector,
    GetBlockNumberResponse, GetBlockTimestampResponse, GetContractAddressResponse,
    LibraryCallRequest, SyscallRequest, SyscallResponse, WriteResponseResult,
};
use blockifier::execution::execution_utils::{write_maybe_relocatable, ReadOnlySegment};
use blockifier::execution::hint_code;

use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::deprecated::calls::execute_library_call;
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
// crates/blockifier/src/execution/deprecated_syscalls/mod.rs:147 (SingleSegmentResponse)
// It is created here because fields in the original structure are private
// so we cannot create it in call_contract_syscall
pub struct SingleSegmentResponse {
    pub(crate) segment: ReadOnlySegment,
}
// crates/blockifier/src/execution/deprecated_syscalls/mod.rs:151 (SyscallResponse for SingleSegmentResponse)
impl SyscallResponse for SingleSegmentResponse {
    fn write(self, vm: &mut VirtualMachine, ptr: &mut Relocatable) -> WriteResponseResult {
        write_maybe_relocatable(vm, ptr, self.segment.length)?;
        write_maybe_relocatable(vm, ptr, self.segment.start_ptr)?;
        Ok(())
    }
}

pub struct CheatableSyscallHandler<'a> {
    pub child: DeprecatedSyscallHintProcessor<'a>,
    pub cheatnet_state: &'a mut CheatnetState,
}

// crates/blockifier/src/execution/deprecated_syscalls/hint_processor.rs:326 (impl ResourceTracker for DeprecatedSyscallHintProcessor)
impl ResourceTracker for CheatableSyscallHandler<'_> {
    fn consumed(&self) -> bool {
        self.child.context.vm_run_resources.consumed()
    }

    fn consume_step(&mut self) {
        self.child.context.vm_run_resources.consume_step();
    }

    fn get_n_steps(&self) -> Option<usize> {
        self.child.context.vm_run_resources.get_n_steps()
    }

    fn run_resources(&self) -> &RunResources {
        self.child.context.vm_run_resources.run_resources()
    }
}

impl HintProcessorLogic for CheatableSyscallHandler<'_> {
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

        self.child
            .execute_hint(vm, exec_scopes, hint_data, constants)
    }
}

impl<'a> CheatableSyscallHandler<'a> {
    pub fn execute_next_syscall_cheated(
        &mut self,
        vm: &mut VirtualMachine,
        ids_data: &HashMap<String, HintReference>,
        ap_tracking: &ApTracking,
    ) -> HintExecutionResult {
        // We peak into the selector without incrementing the pointer as it is done later
        let syscall_selector_pointer = self.child.syscall_ptr;
        let selector = DeprecatedSyscallSelector::try_from(stark_felt_from_ptr_immutable(
            vm,
            &syscall_selector_pointer,
        )?)?;
        self.verify_syscall_ptr(syscall_selector_pointer)?;
        let contract_address = self.child.storage_address;

        if DeprecatedSyscallSelector::GetCallerAddress == selector
            && self.cheatnet_state.address_is_pranked(&contract_address)
        {
            // Increment, since the selector was peeked into before
            self.child.syscall_ptr += 1;
            self.increment_syscall_count(selector);

            let response = get_caller_address(self, contract_address).unwrap();

            response.write(vm, &mut self.child.syscall_ptr)?;

            return Ok(());
        } else if DeprecatedSyscallSelector::GetBlockNumber == selector
            && self.cheatnet_state.address_is_rolled(&contract_address)
        {
            self.child.syscall_ptr += 1;
            self.increment_syscall_count(selector);

            let response = get_block_number(self, contract_address).unwrap();

            response.write(vm, &mut self.child.syscall_ptr)?;

            return Ok(());
        } else if DeprecatedSyscallSelector::GetBlockTimestamp == selector
            && self.cheatnet_state.address_is_warped(&contract_address)
        {
            self.child.syscall_ptr += 1;
            self.increment_syscall_count(selector);

            let response = get_block_timestamp(self, contract_address).unwrap();

            response.write(vm, &mut self.child.syscall_ptr)?;

            return Ok(());
        } else if DeprecatedSyscallSelector::GetSequencerAddress == selector
            && self.cheatnet_state.address_is_elected(&contract_address)
        {
            self.child.syscall_ptr += 1;
            self.increment_syscall_count(selector);

            let response = get_sequencer_address(self, contract_address).unwrap();

            response.write(vm, &mut self.child.syscall_ptr)?;

            return Ok(());
        } else if DeprecatedSyscallSelector::DelegateCall == selector {
            self.child.syscall_ptr += 1;
            self.increment_syscall_count(selector);

            return self.execute_syscall(vm, delegate_call);
        } else if DeprecatedSyscallSelector::LibraryCall == selector {
            self.child.syscall_ptr += 1;
            self.increment_syscall_count(selector);

            return self.execute_syscall(vm, library_call);
        }

        self.child.execute_next_syscall(vm, ids_data, ap_tracking)
    }

    // crates/blockifier/src/execution/deprecated_syscalls/hint_processor.rs:233
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
            &mut CheatableSyscallHandler<'_>,
        ) -> DeprecatedSyscallResult<Response>,
    {
        let request = Request::read(vm, &mut self.child.syscall_ptr)?;

        let response = execute_callback(request, vm, self)?;
        response.write(vm, &mut self.child.syscall_ptr)?;

        Ok(())
    }

    // crates/blockifier/src/execution/deprecated_syscalls/hint_processor.rs:141
    pub fn verify_syscall_ptr(&self, actual_ptr: Relocatable) -> DeprecatedSyscallResult<()> {
        if actual_ptr != self.child.syscall_ptr {
            return Err(DeprecatedSyscallExecutionError::BadSyscallPointer {
                expected_ptr: self.child.syscall_ptr,
                actual_ptr,
            });
        }

        Ok(())
    }

    // crates/blockifier/src/execution/deprecated_syscalls/hint_processor.rs:264
    fn increment_syscall_count(&mut self, selector: DeprecatedSyscallSelector) {
        let syscall_count = self
            .child
            .resources
            .syscall_counter
            .entry(selector)
            .or_default();
        *syscall_count += 1;
    }
}

// blockifier/src/execution/deprecated_syscalls/mod.rs:209 (delegate_call)
pub fn delegate_call(
    request: CallContractRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut CheatableSyscallHandler<'_>,
) -> DeprecatedSyscallResult<SingleSegmentResponse> {
    let call_to_external = true;
    let storage_address = request.contract_address;
    let class_hash = syscall_handler
        .child
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

// blockifier/src/execution/deprecated_syscalls/mod.rs:537 (library_call)
pub fn library_call(
    request: LibraryCallRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut CheatableSyscallHandler<'_>,
) -> DeprecatedSyscallResult<SingleSegmentResponse> {
    let call_to_external = true;
    let retdata_segment = execute_library_call(
        syscall_handler,
        vm,
        request.class_hash,
        None,
        call_to_external,
        request.function_selector,
        request.calldata,
    )?;

    Ok(SingleSegmentResponse {
        segment: retdata_segment,
    })
}

// blockifier/src/execution/deprecated_syscalls/mod.rs:426 (get_caller_address)
pub fn get_caller_address(
    syscall_handler: &mut CheatableSyscallHandler<'_>,
    contract_address: ContractAddress,
) -> DeprecatedSyscallResult<GetContractAddressResponse> {
    Ok(GetContractAddressResponse {
        address: syscall_handler
            .cheatnet_state
            .get_cheated_caller_address(&contract_address)
            .unwrap(),
    })
}

// blockifier/src/execution/deprecated_syscalls/mod.rs:387 (get_block_number)
pub fn get_block_number(
    syscall_handler: &mut CheatableSyscallHandler<'_>,
    contract_address: ContractAddress,
) -> DeprecatedSyscallResult<GetBlockNumberResponse> {
    Ok(GetBlockNumberResponse {
        block_number: BlockNumber(
            syscall_handler
                .cheatnet_state
                .get_cheated_block_number(&contract_address)
                .unwrap()
                .to_u64()
                .unwrap(),
        ),
    })
}

// blockifier/src/execution/deprecated_syscalls/mod.rs:411 (get_block_timestamp)
pub fn get_block_timestamp(
    syscall_handler: &mut CheatableSyscallHandler<'_>,
    contract_address: ContractAddress,
) -> DeprecatedSyscallResult<GetBlockTimestampResponse> {
    Ok(GetBlockTimestampResponse {
        block_timestamp: BlockTimestamp(
            syscall_handler
                .cheatnet_state
                .get_cheated_block_timestamp(&contract_address)
                .unwrap()
                .to_u64()
                .unwrap(),
        ),
    })
}

// blockifier/src/execution/deprecated_syscalls/mod.rs:470 (get_sequencer_address)
type GetSequencerAddressResponse = GetContractAddressResponse;

pub fn get_sequencer_address(
    cheatable_syscall_handler: &mut CheatableSyscallHandler<'_>,
    contract_address: ContractAddress,
) -> DeprecatedSyscallResult<GetSequencerAddressResponse> {
    cheatable_syscall_handler
        .child
        .verify_not_in_validate_mode("get_sequencer_address")?;

    Ok(GetSequencerAddressResponse {
        address: cheatable_syscall_handler
            .cheatnet_state
            .get_cheated_sequencer_address(&contract_address)
            .unwrap(),
    })
}
