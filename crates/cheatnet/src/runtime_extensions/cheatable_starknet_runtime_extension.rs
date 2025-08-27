use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::{
    cheated_syscalls, syscall_hooks,
};
use crate::state::CheatnetState;
use anyhow::Result;
use blockifier::execution::syscalls::hint_processor::OUT_OF_GAS_ERROR;
use blockifier::execution::syscalls::syscall_base::SyscallResult;
use blockifier::execution::syscalls::syscall_executor::SyscallExecutor;
use blockifier::execution::syscalls::vm_syscall_utils::{
    RevertData, SyscallExecutorBaseError, SyscallRequest, SyscallRequestWrapper, SyscallResponse,
    SyscallResponseWrapper, SyscallSelector,
};
use blockifier::execution::{
    common_hints::HintExecutionResult,
    syscalls::hint_processor::{SyscallExecutionError, SyscallHintProcessor},
};
use blockifier::utils::u64_from_usize;
use cairo_vm::{
    types::relocatable::Relocatable,
    vm::{
        errors::{hint_errors::HintError, vm_errors::VirtualMachineError},
        vm_core::VirtualMachine,
    },
};
use runtime::{ExtendedRuntime, ExtensionLogic, StarknetRuntime, SyscallHandlingResult};
use starknet_api::execution_resources::GasAmount;
use starknet_types_core::felt::Felt;

pub struct CheatableStarknetRuntimeExtension<'a> {
    pub cheatnet_state: &'a mut CheatnetState,
}

pub type CheatableStarknetRuntime<'a> = ExtendedRuntime<CheatableStarknetRuntimeExtension<'a>>;

impl<'a> ExtensionLogic for CheatableStarknetRuntimeExtension<'a> {
    type Runtime = StarknetRuntime<'a>;

    fn override_system_call(
        &mut self,
        selector: SyscallSelector,
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
            SyscallSelector::StorageRead => self
                .execute_syscall(
                    syscall_handler,
                    vm,
                    cheated_syscalls::storage_read,
                    SyscallSelector::StorageRead,
                )
                .map(|()| SyscallHandlingResult::Handled),
            SyscallSelector::StorageWrite => self
                .execute_syscall(
                    syscall_handler,
                    vm,
                    cheated_syscalls::storage_write,
                    SyscallSelector::StorageWrite,
                )
                .map(|()| SyscallHandlingResult::Handled),
            _ => Ok(SyscallHandlingResult::Forwarded),
        }
    }

    fn handle_system_call_signal(
        &mut self,
        selector: SyscallSelector,
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

impl CheatableStarknetRuntimeExtension<'_> {
    // crates/blockifier/src/execution/syscalls/vm_syscall_utils.rs:677 (execute_syscall)
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

        let syscall_gas_cost = syscall_handler
            .get_gas_cost_from_selector(&selector)
            .map_err(|error| SyscallExecutorBaseError::GasCost { error, selector })?;

        let SyscallRequestWrapper {
            gas_counter,
            request,
        } = SyscallRequestWrapper::<Request>::read(vm, &mut syscall_handler.syscall_ptr)?;

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
            //  Out of gas failure.
            let out_of_gas_error = Felt::from_hex(OUT_OF_GAS_ERROR)
                .expect("Failed to parse OUT_OF_GAS_ERROR hex string");
            let response: SyscallResponseWrapper<Response> = SyscallResponseWrapper::Failure {
                gas_counter,
                revert_data: RevertData::new_normal(vec![out_of_gas_error]),
            };
            response.write(vm, &mut syscall_handler.syscall_ptr)?;

            return Ok(());
        }

        // Execute.
        let mut remaining_gas = gas_counter - required_gas;

        // TODO(#3681)
        syscall_handler.update_revert_gas_with_next_remaining_gas(GasAmount(remaining_gas));

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
                    revert_data: RevertData::new_normal(data),
                }
            }
            Err(error) => return Err(error.into()),
        };

        response.write(vm, &mut syscall_handler.syscall_ptr)?;

        Ok(())
    }
}
