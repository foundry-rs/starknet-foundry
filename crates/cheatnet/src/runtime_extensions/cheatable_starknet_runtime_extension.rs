use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::{
    cheated_syscalls, syscall_hooks,
};
use crate::state::CheatnetState;
use anyhow::Result;
use blockifier::execution::entry_point::EntryPointExecutionContext;
use blockifier::execution::syscalls::hint_processor::OUT_OF_GAS_ERROR;
use blockifier::execution::syscalls::{
    SyscallRequest, SyscallRequestWrapper, SyscallResponse, SyscallResponseWrapper,
    syscall_base::SyscallResult,
};
use blockifier::execution::{
    common_hints::HintExecutionResult,
    deprecated_syscalls::DeprecatedSyscallSelector,
    syscalls::hint_processor::{SyscallExecutionError, SyscallHintProcessor},
};
use blockifier::utils::u64_from_usize;
use blockifier::versioned_constants::SyscallGasCost;
use cairo_vm::{
    types::relocatable::Relocatable,
    vm::{
        errors::{hint_errors::HintError, vm_errors::VirtualMachineError},
        vm_core::VirtualMachine,
    },
};
use conversions::string::TryFromHexStr;
use runtime::{ExtendedRuntime, ExtensionLogic, StarknetRuntime, SyscallHandlingResult};
use starknet_types_core::felt::Felt;

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
                .map(|()| SyscallHandlingResult::Handled),

            SyscallSelector::CallContract => self
                .execute_syscall(
                    syscall_handler,
                    vm,
                    cheated_syscalls::call_contract_syscall,
                    SyscallSelector::CallContract,
                )
                .map(|()| SyscallHandlingResult::Handled),
            SyscallSelector::LibraryCall => self
                .execute_syscall(
                    syscall_handler,
                    vm,
                    cheated_syscalls::library_call_syscall,
                    SyscallSelector::LibraryCall,
                )
                .map(|()| SyscallHandlingResult::Handled),
            SyscallSelector::Deploy => self
                .execute_syscall(
                    syscall_handler,
                    vm,
                    cheated_syscalls::deploy_syscall,
                    SyscallSelector::Deploy,
                )
                .map(|()| SyscallHandlingResult::Handled),
            SyscallSelector::GetBlockHash => self
                .execute_syscall(
                    syscall_handler,
                    vm,
                    cheated_syscalls::get_block_hash_syscall,
                    SyscallSelector::GetBlockHash,
                )
                .map(|()| SyscallHandlingResult::Handled),
            _ => Ok(SyscallHandlingResult::Forwarded),
        }
    }

    fn handle_system_call_signal(
        &mut self,
        selector: DeprecatedSyscallSelector,
        _vm: &mut VirtualMachine,
        extended_runtime: &mut Self::Runtime,
    ) {
        let syscall_handler = &extended_runtime.hint_handler;
        match selector {
            SyscallSelector::EmitEvent => {
                syscall_hooks::emit_event_hook(syscall_handler, self.cheatnet_state);
            }
            SyscallSelector::SendMessageToL1 => {
                syscall_hooks::send_message_to_l1_syscall_hook(
                    syscall_handler,
                    self.cheatnet_state,
                );
            }
            _ => {}
        }
    }
}

pub fn felt_from_ptr_immutable(
    vm: &VirtualMachine,
    ptr: &Relocatable,
) -> Result<Felt, VirtualMachineError> {
    let felt = vm.get_integer(*ptr)?.into_owned();
    Ok(felt)
}

fn get_syscall_cost(
    syscall_selector: SyscallSelector,
    context: &EntryPointExecutionContext,
) -> SyscallGasCost {
    let gas_costs = context.gas_costs();
    match syscall_selector {
        SyscallSelector::LibraryCall => gas_costs.syscalls.library_call,
        SyscallSelector::CallContract => gas_costs.syscalls.call_contract,
        SyscallSelector::Deploy => gas_costs.syscalls.deploy,
        SyscallSelector::GetExecutionInfo => gas_costs.syscalls.get_execution_info,
        SyscallSelector::GetBlockHash => gas_costs.syscalls.get_block_hash,
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

        let SyscallRequestWrapper {
            gas_counter,
            request,
        } = SyscallRequestWrapper::<Request>::read(vm, &mut syscall_handler.syscall_ptr)?;

        let syscall_gas_cost = get_syscall_cost(selector, syscall_handler.base.context);
        let syscall_gas_cost =
            syscall_gas_cost.get_syscall_cost(u64_from_usize(request.get_linear_factor_length()));
        let syscall_base_cost = syscall_handler
            .base
            .context
            .gas_costs()
            .base
            .syscall_base_gas_cost;

        // Sanity check for preventing underflow.
        assert!(
            syscall_gas_cost >= syscall_base_cost,
            "Syscall gas cost must be greater than base syscall gas cost"
        );

        // Refund `SYSCALL_BASE_GAS_COST` as it was pre-charged.
        let required_gas = syscall_gas_cost - syscall_base_cost;

        if gas_counter < required_gas {
            //  Out of gas failure.
            let out_of_gas_error = TryFromHexStr::try_from_hex_str(OUT_OF_GAS_ERROR)
                .map_err(SyscallExecutionError::from)?;
            let response: SyscallResponseWrapper<Response> = SyscallResponseWrapper::Failure {
                gas_counter,
                error_data: vec![out_of_gas_error],
            };
            response.write(vm, &mut syscall_handler.syscall_ptr)?;

            return Ok(());
        }

        // Execute.
        let mut remaining_gas = gas_counter - required_gas;
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
            Err(SyscallExecutionError::Revert { error_data: data }) => {
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
