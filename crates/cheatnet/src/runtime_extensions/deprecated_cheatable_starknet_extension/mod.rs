use crate::state::CheatnetState;
use blockifier::execution::common_hints::HintExecutionResult;
use blockifier::execution::deprecated_syscalls::hint_processor::{
    DeprecatedSyscallExecutionError, DeprecatedSyscallHintProcessor,
};
use blockifier::execution::deprecated_syscalls::{
    CallContractRequest, DeployRequest, DeployResponse, DeprecatedSyscallResult,
    DeprecatedSyscallSelector, GetBlockNumberResponse, GetBlockTimestampResponse,
    GetContractAddressResponse, LibraryCallRequest, SyscallRequest, SyscallResponse,
    WriteResponseResult,
};
use blockifier::execution::entry_point::{CallEntryPoint, CallType, ConstructorContext};
use blockifier::execution::execution_utils::{
    execute_deployment, write_maybe_relocatable, ReadOnlySegment,
};
use conversions::FromConv;

use ::runtime::SyscallHandlingResult;
use cairo_vm::types::relocatable::{MaybeRelocatable, Relocatable};
use cairo_vm::vm::errors::hint_errors::HintError;
use cairo_vm::vm::vm_core::VirtualMachine;
use num_traits::ToPrimitive;
use starknet_api::block::{BlockNumber, BlockTimestamp};
use starknet_api::contract_class::EntryPointType;
use starknet_api::core::{
    calculate_contract_address, ClassHash, ContractAddress, EntryPointSelector,
};
use starknet_api::transaction::fields::Calldata;
use starknet_types_core::felt::Felt;

use self::runtime::{
    DeprecatedExtendedRuntime, DeprecatedExtensionLogic, DeprecatedStarknetRuntime,
};

use super::call_to_blockifier_runtime_extension::execution::entry_point::execute_call_entry_point;
use super::call_to_blockifier_runtime_extension::execution::syscall_hooks;

pub mod runtime;

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

pub struct DeprecatedCheatableStarknetRuntimeExtension<'a> {
    pub cheatnet_state: &'a mut CheatnetState,
}

pub type DeprecatedCheatableStarknetRuntime<'a> =
    DeprecatedExtendedRuntime<DeprecatedCheatableStarknetRuntimeExtension<'a>>;

impl<'a> DeprecatedExtensionLogic for DeprecatedCheatableStarknetRuntimeExtension<'a> {
    type Runtime = DeprecatedStarknetRuntime<'a>;

    fn override_system_call(
        &mut self,
        selector: DeprecatedSyscallSelector,
        vm: &mut VirtualMachine,
        extended_runtime: &mut Self::Runtime,
    ) -> Result<SyscallHandlingResult, HintError> {
        let syscall_handler = &mut extended_runtime.hint_handler;
        let contract_address = syscall_handler.storage_address;
        match selector {
            DeprecatedSyscallSelector::GetCallerAddress => {
                if let Some(caller_address) = self
                    .cheatnet_state
                    .get_cheated_caller_address(contract_address)
                {
                    // Increment, since the selector was peeked into before
                    syscall_handler.syscall_ptr += 1;
                    increment_syscall_count(syscall_handler, selector);

                    let response = GetContractAddressResponse {
                        address: caller_address,
                    };

                    response.write(vm, &mut syscall_handler.syscall_ptr)?;
                    Ok(SyscallHandlingResult::Handled)
                } else {
                    Ok(SyscallHandlingResult::Forwarded)
                }
            }
            DeprecatedSyscallSelector::GetBlockNumber => {
                if let Some(block_number) = self
                    .cheatnet_state
                    .get_cheated_block_number(contract_address)
                {
                    syscall_handler.syscall_ptr += 1;
                    increment_syscall_count(syscall_handler, selector);

                    let response = GetBlockNumberResponse {
                        block_number: BlockNumber(block_number.to_u64().unwrap()),
                    };

                    response.write(vm, &mut syscall_handler.syscall_ptr)?;
                    Ok(SyscallHandlingResult::Handled)
                } else {
                    Ok(SyscallHandlingResult::Forwarded)
                }
            }
            DeprecatedSyscallSelector::GetBlockTimestamp => {
                if let Some(block_timestamp) = self
                    .cheatnet_state
                    .get_cheated_block_timestamp(contract_address)
                {
                    syscall_handler.syscall_ptr += 1;
                    increment_syscall_count(syscall_handler, selector);

                    let response = GetBlockTimestampResponse {
                        block_timestamp: BlockTimestamp(block_timestamp.to_u64().unwrap()),
                    };

                    response.write(vm, &mut syscall_handler.syscall_ptr)?;
                    Ok(SyscallHandlingResult::Handled)
                } else {
                    Ok(SyscallHandlingResult::Forwarded)
                }
            }
            DeprecatedSyscallSelector::GetSequencerAddress => {
                if let Some(sequencer_address) = self
                    .cheatnet_state
                    .get_cheated_sequencer_address(contract_address)
                {
                    syscall_handler.syscall_ptr += 1;
                    increment_syscall_count(syscall_handler, selector);

                    syscall_handler.verify_not_in_validate_mode("get_sequencer_address")?;

                    let response = GetContractAddressResponse {
                        address: sequencer_address,
                    };

                    response.write(vm, &mut syscall_handler.syscall_ptr)?;

                    Ok(SyscallHandlingResult::Handled)
                } else {
                    Ok(SyscallHandlingResult::Forwarded)
                }
            }
            DeprecatedSyscallSelector::DelegateCall => {
                syscall_handler.syscall_ptr += 1;
                increment_syscall_count(syscall_handler, selector);

                self.execute_syscall(vm, delegate_call, syscall_handler)?;
                Ok(SyscallHandlingResult::Handled)
            }
            DeprecatedSyscallSelector::LibraryCall => {
                syscall_handler.syscall_ptr += 1;
                increment_syscall_count(syscall_handler, selector);

                self.execute_syscall(vm, library_call, syscall_handler)?;
                Ok(SyscallHandlingResult::Handled)
            }
            DeprecatedSyscallSelector::CallContract => {
                syscall_handler.syscall_ptr += 1;
                increment_syscall_count(syscall_handler, selector);

                self.execute_syscall(vm, call_contract, syscall_handler)?;
                Ok(SyscallHandlingResult::Handled)
            }
            DeprecatedSyscallSelector::Deploy => {
                syscall_handler.syscall_ptr += 1;
                increment_syscall_count(syscall_handler, selector);

                self.execute_syscall(vm, deploy, syscall_handler)?;
                Ok(SyscallHandlingResult::Handled)
            }
            _ => Ok(SyscallHandlingResult::Forwarded),
        }
    }

    fn post_syscall_hook(
        &mut self,
        selector: &DeprecatedSyscallSelector,
        extended_runtime: &mut Self::Runtime,
    ) {
        let syscall_handler = &extended_runtime.hint_handler;
        match selector {
            DeprecatedSyscallSelector::EmitEvent => {
                syscall_hooks::emit_event_hook(syscall_handler, self.cheatnet_state);
            }
            DeprecatedSyscallSelector::SendMessageToL1 => {
                syscall_hooks::send_message_to_l1_syscall_hook(
                    syscall_handler,
                    self.cheatnet_state,
                );
            }
            _ => {}
        }
    }
}

impl DeprecatedCheatableStarknetRuntimeExtension<'_> {
    // crates/blockifier/src/execution/deprecated_syscalls/hint_processor.rs:233
    fn execute_syscall<Request, Response, ExecuteCallback>(
        &mut self,
        vm: &mut VirtualMachine,
        execute_callback: ExecuteCallback,
        syscall_handler: &mut DeprecatedSyscallHintProcessor,
    ) -> HintExecutionResult
    where
        Request: SyscallRequest,
        Response: SyscallResponse,
        ExecuteCallback: FnOnce(
            Request,
            &mut VirtualMachine,
            &mut DeprecatedSyscallHintProcessor,
            &mut CheatnetState,
        ) -> DeprecatedSyscallResult<Response>,
    {
        let request = Request::read(vm, &mut syscall_handler.syscall_ptr)?;

        let response = execute_callback(request, vm, syscall_handler, self.cheatnet_state)?;
        response.write(vm, &mut syscall_handler.syscall_ptr)?;

        Ok(())
    }
}

// crates/blockifier/src/execution/deprecated_syscalls/hint_processor.rs:264
fn increment_syscall_count(
    syscall_handler: &mut DeprecatedSyscallHintProcessor,
    selector: DeprecatedSyscallSelector,
) {
    let syscall_count = syscall_handler.syscall_counter.entry(selector).or_default();
    *syscall_count += 1;
}

//blockifier/src/execution/deprecated_syscalls/mod.rs:303 (deploy)
fn deploy(
    request: DeployRequest,
    _vm: &mut VirtualMachine,
    syscall_handler: &mut DeprecatedSyscallHintProcessor<'_>,
    _cheatnet_state: &mut CheatnetState,
) -> DeprecatedSyscallResult<DeployResponse> {
    let deployer_address = syscall_handler.storage_address;
    let deployer_address_for_calculation = if request.deploy_from_zero {
        ContractAddress::default()
    } else {
        deployer_address
    };
    let deployed_contract_address = calculate_contract_address(
        request.contract_address_salt,
        request.class_hash,
        &request.constructor_calldata,
        deployer_address_for_calculation,
    )?;

    let ctor_context = ConstructorContext {
        class_hash: request.class_hash,
        code_address: Some(deployed_contract_address),
        storage_address: deployed_contract_address,
        caller_address: deployer_address,
    };
    let call_info = execute_deployment(
        syscall_handler.state,
        syscall_handler.context,
        ctor_context,
        request.constructor_calldata,
        //TODO MOCK
        &mut 420_000_u64,
    )?;
    syscall_handler.inner_calls.push(call_info);

    Ok(DeployResponse {
        contract_address: deployed_contract_address,
    })
}

//blockifier/src/execution/deprecated_syscalls/mod.rs:182 (call_contract)
fn call_contract(
    request: CallContractRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut DeprecatedSyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
) -> DeprecatedSyscallResult<SingleSegmentResponse> {
    let storage_address = request.contract_address;
    // Check that the call is legal if in Validate execution mode.
    if syscall_handler.is_validate_mode() && syscall_handler.storage_address != storage_address {
        return Err(
            DeprecatedSyscallExecutionError::InvalidSyscallInExecutionMode {
                syscall_name: "call_contract".to_string(),
                execution_mode: syscall_handler.execution_mode(),
            },
        );
    }
    let mut entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(storage_address),
        entry_point_type: EntryPointType::External,
        entry_point_selector: request.function_selector,
        calldata: request.calldata,
        storage_address,
        caller_address: syscall_handler.storage_address,
        call_type: CallType::Call,
        initial_gas: syscall_handler
            .context
            .gas_costs()
            .base
            .default_initial_gas_cost,
    };
    let retdata_segment =
        execute_inner_call(&mut entry_point, vm, syscall_handler, cheatnet_state)?;

    Ok(SingleSegmentResponse {
        segment: retdata_segment,
    })
}

// blockifier/src/execution/deprecated_syscalls/mod.rs:209 (delegate_call)
fn delegate_call(
    request: CallContractRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut DeprecatedSyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
) -> DeprecatedSyscallResult<SingleSegmentResponse> {
    let call_to_external = true;
    let storage_address = request.contract_address;
    let class_hash = syscall_handler.state.get_class_hash_at(storage_address)?;
    let retdata_segment = execute_library_call(
        syscall_handler,
        cheatnet_state,
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
fn library_call(
    request: LibraryCallRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut DeprecatedSyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
) -> DeprecatedSyscallResult<SingleSegmentResponse> {
    let call_to_external = true;
    let retdata_segment = execute_library_call(
        syscall_handler,
        cheatnet_state,
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

// blockifier/src/execution/deprecated_syscalls/hint_processor.rs:393 (execute_inner_call)
fn execute_inner_call(
    call: &mut CallEntryPoint,
    vm: &mut VirtualMachine,
    syscall_handler: &mut DeprecatedSyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
) -> DeprecatedSyscallResult<ReadOnlySegment> {
    // region: Modified blockifier code
    let call_info = execute_call_entry_point(
        call,
        syscall_handler.state,
        cheatnet_state,
        // syscall_handler.resources,
        syscall_handler.context,
    )?;
    // endregion

    let retdata = &call_info.execution.retdata.0;
    let retdata: Vec<MaybeRelocatable> = retdata
        .iter()
        .map(|&x| MaybeRelocatable::from(Felt::from_(x)))
        .collect();
    let retdata_segment_start_ptr = syscall_handler.read_only_segments.allocate(vm, &retdata)?;

    syscall_handler.inner_calls.push(call_info);
    Ok(ReadOnlySegment {
        start_ptr: retdata_segment_start_ptr,
        length: retdata.len(),
    })
}

// blockifier/src/execution/deprecated_syscalls/hint_processor.rs:409 (execute_library_call)
#[allow(clippy::too_many_arguments)]
fn execute_library_call(
    syscall_handler: &mut DeprecatedSyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
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
        storage_address: syscall_handler.storage_address,
        caller_address: syscall_handler.caller_address,
        call_type: CallType::Delegate,
        initial_gas: syscall_handler
            .context
            .gas_costs()
            .base
            .default_initial_gas_cost,
    };

    execute_inner_call(&mut entry_point, vm, syscall_handler, cheatnet_state)
}
