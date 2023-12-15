use std::marker::PhantomData;

use blockifier::execution::{
    deprecated_syscalls::DeprecatedSyscallSelector,
    execution_utils::{stark_felt_from_ptr, ReadOnlySegment},
    syscalls::{
        hint_processor::{read_felt_array, SyscallExecutionError, SyscallHintProcessor},
        SyscallRequest, SyscallResponse, SyscallResponseWrapper, SyscallResult,
    },
};

use cairo_felt::Felt252;
use cairo_vm::{
    types::relocatable::Relocatable,
    vm::{errors::hint_errors::HintError, vm_core::VirtualMachine},
};
use conversions::{FromConv, IntoConv};
use num_traits::ToPrimitive;
use runtime::{ExtendedRuntime, ExtensionLogic, SyscallHandlingResult, SyscallPtrAccess};
use starknet_api::{core::ContractAddress, hash::StarkFelt};

use crate::state::{BlockifierState, CheatnetState};

use crate::runtime_extensions::call_to_blockifier_runtime_extension::{
    execution::cheated_syscalls::SingleSegmentResponse,
    rpc::{call_contract, CallContractFailure, CallContractOutput, CallContractResult},
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
            // We execute contract calls with modified blockifier
            // This is redirected to drop ForgeRuntimeExtension
            // and to ensure it behaves closely to real on-chain environment
            DeprecatedSyscallSelector::CallContract => {
                let call_args = CallContractArgs::read(vm, extended_runtime.get_mut_syscall_ptr())?;
                let cheatable_starknet_runtime = &mut extended_runtime.extended_runtime;
                let cheatnet_state: &mut _ = cheatable_starknet_runtime.extension.cheatnet_state;
                let syscall_handler = &mut cheatable_starknet_runtime.extended_runtime.hint_handler;
                let mut blockifier_state = BlockifierState::from(syscall_handler.state);

                let call_result =
                    execute_call_contract(&mut blockifier_state, cheatnet_state, &call_args);
                write_call_contract_response(
                    syscall_handler,
                    cheatnet_state,
                    vm,
                    &call_args,
                    call_result,
                )?;
                Ok(SyscallHandlingResult::Handled(()))
            }
            _ => Ok(SyscallHandlingResult::Forwarded),
        }
    }
}
struct CallContractArgs {
    _selector: Felt252,
    gas_counter: u64,
    contract_address: ContractAddress,
    entry_point_selector: Felt252,
    calldata: Vec<Felt252>,
}

impl SyscallRequest for CallContractArgs {
    fn read(vm: &VirtualMachine, ptr: &mut Relocatable) -> SyscallResult<CallContractArgs> {
        let selector = stark_felt_from_ptr(vm, ptr)?.into_();
        let gas_counter = Felt252::from_(stark_felt_from_ptr(vm, ptr)?)
            .to_u64()
            .unwrap();

        let contract_address = stark_felt_from_ptr(vm, ptr)?.into_();
        let entry_point_selector = stark_felt_from_ptr(vm, ptr)?.into_();

        let calldata = read_felt_array::<SyscallExecutionError>(vm, ptr)?
            .iter()
            .map(|el| (*el).into_())
            .collect();

        Ok(CallContractArgs {
            _selector: selector,
            gas_counter,
            contract_address,
            entry_point_selector,
            calldata,
        })
    }
}

fn execute_call_contract(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    call_args: &CallContractArgs,
) -> CallContractOutput {
    call_contract(
        blockifier_state,
        cheatnet_state,
        &call_args.contract_address,
        &call_args.entry_point_selector,
        &call_args.calldata,
    )
    .unwrap_or_else(|err| panic!("Transaction execution error: {err}"))
}

fn write_call_contract_response(
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    vm: &mut VirtualMachine,
    call_args: &CallContractArgs,
    call_output: CallContractOutput,
) -> Result<(), HintError> {
    let response_wrapper: SyscallResponseWrapper<SingleSegmentResponse> = match call_output.result {
        CallContractResult::Success { ret_data, .. } => {
            let memory_segment_start_ptr = syscall_handler
                .read_only_segments
                .allocate(vm, &ret_data.iter().map(Into::into).collect())?;

            // add execution resources used by call to all used resources
            cheatnet_state
                .used_resources
                .extend(&call_output.used_resources);

            SyscallResponseWrapper::Success {
                gas_counter: call_args.gas_counter,
                response: SingleSegmentResponse {
                    segment: ReadOnlySegment {
                        start_ptr: memory_segment_start_ptr,
                        length: ret_data.len(),
                    },
                },
            }
        }
        CallContractResult::Failure(failure_type) => match failure_type {
            CallContractFailure::Panic { panic_data, .. } => SyscallResponseWrapper::Failure {
                gas_counter: call_args.gas_counter,
                error_data: panic_data
                    .iter()
                    .map(|el| StarkFelt::from_(el.clone()))
                    .collect(),
            },
            CallContractFailure::Error { msg, .. } => {
                return Err(HintError::CustomHint(Box::from(msg)))
            }
        },
    };

    response_wrapper.write(vm, &mut syscall_handler.syscall_ptr)?;

    Ok(())
}
