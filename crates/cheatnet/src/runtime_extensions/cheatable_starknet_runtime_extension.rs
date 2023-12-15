use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::{
    cheated_syscalls, syscall_hooks,
};
use crate::state::CheatnetState;
use anyhow::Result;
use blockifier::execution::syscalls::{
    SyscallRequest, SyscallRequestWrapper, SyscallResponse, SyscallResponseWrapper, SyscallResult,
};
use blockifier::execution::{
    common_hints::HintExecutionResult,
    deprecated_syscalls::DeprecatedSyscallSelector,
    execution_utils::felt_to_stark_felt,
    syscalls::hint_processor::{SyscallExecutionError, SyscallHintProcessor},
};
use blockifier::{abi::constants, execution::syscalls::hint_processor::OUT_OF_GAS_ERROR};
use cairo_felt::Felt252;
use cairo_vm::{
    types::relocatable::Relocatable,
    vm::{
        errors::{hint_errors::HintError, vm_errors::VirtualMachineError},
        vm_core::VirtualMachine,
    },
};
use runtime::{ExtendedRuntime, ExtensionLogic, StarknetRuntime, SyscallHandlingResult};
use starknet_api::hash::StarkFelt;

pub type SyscallSelector = DeprecatedSyscallSelector;

pub struct CheatableStarknetRuntimeExtension<'a> {
    pub cheatnet_state: &'a mut CheatnetState,
}

pub type CheatableStarknetRuntime<'a> = ExtendedRuntime<CheatableStarknetRuntimeExtension<'a>>;

impl<'a> ExtensionLogic for CheatableStarknetRuntimeExtension<'a> {
    type Runtime = StarknetRuntime<'a>;

    fn override_system_call(
        &mut self,
        selector: DeprecatedSyscallSelector,
        vm: &mut VirtualMachine,
        extended_runtime: &mut Self::Runtime,
    ) -> Result<SyscallHandlingResult, HintError> {
        let syscall_handler = &mut extended_runtime.hint_handler;

        match selector {
            SyscallSelector::GetExecutionInfo => self
                .execute_syscall(
                    syscall_handler,
                    vm,
                    cheated_syscalls::get_execution_info_syscall,
                    SyscallSelector::GetExecutionInfo,
                )
                .map(|()| SyscallHandlingResult::Handled(())),

            SyscallSelector::CallContract => self
                .execute_syscall(
                    syscall_handler,
                    vm,
                    cheated_syscalls::call_contract_syscall,
                    SyscallSelector::CallContract,
                )
                .map(|()| SyscallHandlingResult::Handled(())),
            SyscallSelector::LibraryCall => self
                .execute_syscall(
                    syscall_handler,
                    vm,
                    cheated_syscalls::library_call_syscall,
                    SyscallSelector::LibraryCall,
                )
                .map(|()| SyscallHandlingResult::Handled(())),
            SyscallSelector::Deploy => self
                .execute_syscall(
                    syscall_handler,
                    vm,
                    cheated_syscalls::deploy_syscall,
                    SyscallSelector::Deploy,
                )
                .map(|()| SyscallHandlingResult::Handled(())),
            _ => Ok(SyscallHandlingResult::Forwarded),
        }
    }

    fn post_syscall_hook(
        &mut self,
        selector: &DeprecatedSyscallSelector,
        extended_runtime: &mut Self::Runtime,
    ) {
        let syscall_handler = &mut extended_runtime.hint_handler;
        if let SyscallSelector::EmitEvent = selector {
            syscall_hooks::emit_event_hook(syscall_handler, self.cheatnet_state);
        }
    }
}

pub fn stark_felt_from_ptr_immutable(
    vm: &VirtualMachine,
    ptr: &Relocatable,
) -> Result<StarkFelt, VirtualMachineError> {
    Ok(felt_to_stark_felt(&felt_from_ptr_immutable(vm, ptr)?))
}

pub fn felt_from_ptr_immutable(
    vm: &VirtualMachine,
    ptr: &Relocatable,
) -> Result<Felt252, VirtualMachineError> {
    let felt = vm.get_integer(*ptr)?.into_owned();
    Ok(felt)
}

fn get_syscall_cost(syscall_selector: SyscallSelector) -> u64 {
    match syscall_selector {
        SyscallSelector::LibraryCall | SyscallSelector::CallContract => {
            constants::CALL_CONTRACT_GAS_COST
        }
        SyscallSelector::Deploy => constants::DEPLOY_GAS_COST,
        SyscallSelector::GetExecutionInfo => constants::GET_EXECUTION_INFO_GAS_COST,
        _ => unreachable!("Syscall has no associated cost"),
    }
}

impl CheatableStarknetRuntimeExtension<'_> {
    // crates/blockifier/src/execution/syscalls/hint_processor.rs:280 (SyscallHintProcessor::execute_syscall)
    fn execute_syscall<Request, Response, ExecuteCallback>(
        &mut self,
        syscall_handler: &mut SyscallHintProcessor,
        vm: &mut VirtualMachine,
        execute_callback: ExecuteCallback,
        selector: SyscallSelector,
    ) -> HintExecutionResult
    where
        Request: SyscallRequest + std::fmt::Debug,
        Response: SyscallResponse + std::fmt::Debug,
        ExecuteCallback: FnOnce(
            Request,
            &mut VirtualMachine,
            &mut SyscallHintProcessor<'_>,
            &mut CheatnetState,
            &mut u64, // Remaining gas.
        ) -> SyscallResult<Response>,
    {
        // Increment, since the selector was peeked into before
        syscall_handler.syscall_ptr += 1;
        syscall_handler.increment_syscall_count_by(&selector, 1);
        let base_gas_cost = get_syscall_cost(selector);

        let SyscallRequestWrapper {
            gas_counter,
            request,
        } = SyscallRequestWrapper::<Request>::read(vm, &mut syscall_handler.syscall_ptr)?;

        if gas_counter < base_gas_cost {
            //  Out of gas failure.
            let out_of_gas_error =
                StarkFelt::try_from(OUT_OF_GAS_ERROR).map_err(SyscallExecutionError::from)?;
            let response: SyscallResponseWrapper<Response> = SyscallResponseWrapper::Failure {
                gas_counter,
                error_data: vec![out_of_gas_error],
            };
            response.write(vm, &mut syscall_handler.syscall_ptr)?;

            return Ok(());
        }

        // Execute.
        let mut remaining_gas = gas_counter - base_gas_cost;
        let original_response = execute_callback(
            request,
            vm,
            syscall_handler,
            self.cheatnet_state,
            &mut remaining_gas,
        );
        let response = match original_response {
            Ok(response) => SyscallResponseWrapper::Success {
                gas_counter: remaining_gas,
                response,
            },
            Err(SyscallExecutionError::SyscallError { error_data: data }) => {
                SyscallResponseWrapper::Failure {
                    gas_counter: remaining_gas,
                    error_data: data,
                }
            }
            Err(error) => return Err(error.into()),
        };

        response.write(vm, &mut syscall_handler.syscall_ptr)?;

        Ok(())
    }
}
