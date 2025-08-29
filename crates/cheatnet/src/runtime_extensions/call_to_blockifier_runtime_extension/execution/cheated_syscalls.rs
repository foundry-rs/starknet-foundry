use super::calls::{execute_inner_call, execute_library_call};
use super::execution_info::get_cheated_exec_info_ptr;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::CheatnetState;
use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::entry_point::execute_constructor_entry_point;
use blockifier::context::TransactionContext;
use blockifier::execution::common_hints::ExecutionMode;
use blockifier::execution::execution_utils::ReadOnlySegment;
use blockifier::execution::syscalls::hint_processor::{
    INVALID_ARGUMENT, SyscallExecutionError, SyscallHintProcessor,
};
use blockifier::execution::syscalls::syscall_base::SyscallResult;
use blockifier::execution::syscalls::vm_syscall_utils::{
    CallContractRequest, CallContractResponse, DeployRequest, DeployResponse, EmptyRequest,
    GetBlockHashRequest, GetBlockHashResponse, GetExecutionInfoResponse, LibraryCallRequest,
    LibraryCallResponse, MetaTxV0Request, MetaTxV0Response, StorageReadRequest,
    StorageReadResponse, StorageWriteRequest, StorageWriteResponse, SyscallSelector,
    TryExtractRevert,
};
use blockifier::execution::{call_info::CallInfo, entry_point::ConstructorContext};
use blockifier::state::errors::StateError;
use blockifier::transaction::objects::{
    CommonAccountFields, DeprecatedTransactionInfo, TransactionInfo,
};
use blockifier::{
    execution::entry_point::{
        CallEntryPoint, CallType, EntryPointExecutionContext, EntryPointExecutionResult,
    },
    state::state_api::State,
};
use blockifier::{
    execution::execution_utils::update_remaining_gas,
    execution::syscalls::hint_processor::create_retdata_segment,
};
use cairo_vm::Felt252;
use cairo_vm::vm::vm_core::VirtualMachine;
use conversions::string::TryFromHexStr;
use runtime::starknet::constants::TEST_ADDRESS;
use starknet_api::abi::abi_utils::selector_from_name;
use starknet_api::core::EntryPointSelector;
use starknet_api::transaction::constants::EXECUTE_ENTRY_POINT_NAME;
use starknet_api::transaction::fields::TransactionSignature;
use starknet_api::transaction::{TransactionHasher, TransactionOptions, signed_tx_version};
use starknet_api::{
    contract_class::EntryPointType,
    core::{ClassHash, ContractAddress, Nonce, calculate_contract_address},
    transaction::{
        InvokeTransactionV0, TransactionVersion,
        fields::{Calldata, Fee},
    },
};
use std::sync::Arc;

#[expect(clippy::result_large_err)]
pub fn get_execution_info_syscall(
    _request: EmptyRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    _remaining_gas: &mut u64,
) -> SyscallResult<GetExecutionInfoResponse> {
    let execution_info_ptr = syscall_handler.get_or_allocate_execution_info_segment(vm)?;

    let cheated_data = cheatnet_state.get_cheated_data(syscall_handler.storage_address());

    let ptr_cheated_exec_info = get_cheated_exec_info_ptr(vm, execution_info_ptr, &cheated_data);

    Ok(GetExecutionInfoResponse {
        execution_info_ptr: ptr_cheated_exec_info,
    })
}

// blockifier/src/execution/syscalls/mod.rs:222 (deploy_syscall)
#[expect(clippy::result_large_err)]
pub fn deploy_syscall(
    request: DeployRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    remaining_gas: &mut u64,
) -> SyscallResult<DeployResponse> {
    // Increment the Deploy syscall's linear cost counter by the number of elements in the
    // constructor calldata
    syscall_handler.increment_linear_factor_by(
        &SyscallSelector::Deploy,
        request.constructor_calldata.0.len(),
    );

    // region: Modified blockifier code
    let deployer_address = syscall_handler.storage_address();
    // endregion
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
        syscall_handler.base.state,
        cheatnet_state,
        syscall_handler.base.context,
        &ctor_context,
        request.constructor_calldata,
        *remaining_gas,
    )?;

    let constructor_retdata =
        create_retdata_segment(vm, syscall_handler, &call_info.execution.retdata.0)?;
    update_remaining_gas(remaining_gas, &call_info);

    syscall_handler.base.inner_calls.push(call_info);

    Ok(DeployResponse {
        contract_address: deployed_contract_address,
        constructor_retdata,
    })
}

// blockifier/src/execution/execution_utils.rs:217 (execute_deployment)
#[expect(clippy::result_large_err)]
pub fn execute_deployment(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    context: &mut EntryPointExecutionContext,
    ctor_context: &ConstructorContext,
    constructor_calldata: Calldata,
    remaining_gas: u64,
) -> EntryPointExecutionResult<CallInfo> {
    // Address allocation in the state is done before calling the constructor, so that it is
    // visible from it.
    let deployed_contract_address = ctor_context.storage_address;
    let current_class_hash = state.get_class_hash_at(deployed_contract_address)?;
    if current_class_hash != ClassHash::default() {
        return Err(StateError::UnavailableContractAddress(deployed_contract_address).into());
    }

    state.set_class_hash_at(deployed_contract_address, ctor_context.class_hash)?;

    let call_info = execute_constructor_entry_point(
        state,
        cheatnet_state,
        context,
        ctor_context,
        constructor_calldata,
        remaining_gas,
    )?;

    Ok(call_info)
}

// blockifier/src/execution/syscalls/mod.rs:407 (library_call)
#[expect(clippy::result_large_err)]
pub fn library_call_syscall(
    request: LibraryCallRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    remaining_gas: &mut u64,
) -> SyscallResult<LibraryCallResponse> {
    let call_to_external = true;
    let retdata_segment = execute_library_call(
        syscall_handler,
        cheatnet_state,
        vm,
        request.class_hash,
        call_to_external,
        request.function_selector,
        request.calldata,
        remaining_gas,
    )
    .map_err(|error| match error {
        SyscallExecutionError::Revert { .. } => error,
        _ => error.as_lib_call_execution_error(
            request.class_hash,
            syscall_handler.storage_address(),
            request.function_selector,
        ),
    })?;

    Ok(LibraryCallResponse {
        segment: retdata_segment,
    })
}

// blockifier/src/execution/syscalls/mod.rs:157 (call_contract)
#[expect(clippy::result_large_err)]
pub fn call_contract_syscall(
    request: CallContractRequest,
    vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    remaining_gas: &mut u64,
) -> SyscallResult<CallContractResponse> {
    let storage_address = request.contract_address;
    let class_hash = syscall_handler
        .base
        .state
        .get_class_hash_at(storage_address)?;
    let selector = request.function_selector;

    let mut entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(storage_address),
        entry_point_type: EntryPointType::External,
        entry_point_selector: selector,
        calldata: request.calldata,
        storage_address,
        caller_address: syscall_handler.storage_address(),
        call_type: CallType::Call,
        initial_gas: *remaining_gas,
    };
    let retdata_segment = execute_inner_call(
        &mut entry_point,
        vm,
        syscall_handler,
        cheatnet_state,
        remaining_gas,
    )
    .map_err(|error| match error {
        SyscallExecutionError::Revert { .. } => error,
        _ => error.as_call_contract_execution_error(class_hash, storage_address, selector),
    })?;

    // region: Modified blockifier code
    Ok(CallContractResponse {
        segment: retdata_segment,
    })
    // endregion
}

// blockifier/src/execution/syscalls/hint_processor.rs:637 (meta_tx_v0)
#[allow(clippy::result_large_err)]
pub fn meta_tx_v0_syscall(
    request: MetaTxV0Request,
    vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    remaining_gas: &mut u64,
) -> SyscallResult<MetaTxV0Response> {
    let storage_address = request.contract_address;
    let selector = request.entry_point_selector;

    // region: Modified blockifier code
    let retdata_segment = meta_tx_v0(
        syscall_handler,
        vm,
        cheatnet_state,
        storage_address,
        selector,
        request.calldata,
        request.signature,
        remaining_gas,
    )?;
    // endregion

    Ok(MetaTxV0Response {
        segment: retdata_segment,
    })
}

// blockifier/src/execution/syscalls/syscall_base.rs:278 (meta_tx_v0)
#[allow(clippy::result_large_err, clippy::too_many_arguments)]
fn meta_tx_v0(
    syscall_handler: &mut SyscallHintProcessor<'_>,
    vm: &mut VirtualMachine,
    cheatnet_state: &mut CheatnetState,
    contract_address: ContractAddress,
    entry_point_selector: EntryPointSelector,
    calldata: Calldata,
    signature: TransactionSignature,
    remaining_gas: &mut u64,
) -> SyscallResult<ReadOnlySegment> {
    syscall_handler.increment_linear_factor_by(&SyscallSelector::MetaTxV0, calldata.0.len());

    if syscall_handler.base.context.execution_mode == ExecutionMode::Validate {
        //region: Modified blockifier code
        unreachable!(
            "`ExecutionMode::Validate` should never occur as execution mode is hardcoded to `Execute`"
        );
        // endregion
    }

    if entry_point_selector != selector_from_name(EXECUTE_ENTRY_POINT_NAME) {
        return Err(SyscallExecutionError::Revert {
            error_data: vec![Felt252::from_hex(INVALID_ARGUMENT).unwrap()],
        });
    }

    let mut entry_point = CallEntryPoint {
        class_hash: None,
        code_address: Some(contract_address),
        entry_point_type: EntryPointType::External,
        entry_point_selector,
        calldata: calldata.clone(),
        storage_address: contract_address,
        caller_address: ContractAddress::default(),
        call_type: CallType::Call,
        // NOTE: this value might be overridden later on.
        initial_gas: *remaining_gas,
    };

    let old_tx_context = syscall_handler.base.context.tx_context.clone();
    let only_query = old_tx_context.tx_info.only_query();

    // Compute meta-transaction hash.
    let transaction_hash = InvokeTransactionV0 {
        max_fee: Fee(0),
        signature: signature.clone(),
        contract_address,
        entry_point_selector,
        calldata,
    }
    .calculate_transaction_hash(
        &syscall_handler
            .base
            .context
            .tx_context
            .block_context
            .chain_info()
            .chain_id,
        &signed_tx_version(
            &TransactionVersion::ZERO,
            &TransactionOptions { only_query },
        ),
    )?;

    let class_hash = syscall_handler
        .base
        .state
        .get_class_hash_at(contract_address)?;

    // Replace `tx_context`.
    let new_tx_info = TransactionInfo::Deprecated(DeprecatedTransactionInfo {
        common_fields: CommonAccountFields {
            transaction_hash,
            version: TransactionVersion::ZERO,
            signature,
            nonce: Nonce(0.into()),
            sender_address: contract_address,
            only_query,
        },
        max_fee: Fee(0),
    });
    syscall_handler.base.context.tx_context = Arc::new(TransactionContext {
        block_context: old_tx_context.block_context.clone(),
        tx_info: new_tx_info,
    });

    // region: Modified blockifier code
    // No error should be propagated until we restore the old `tx_context`.
    let retdata_segment = execute_inner_call(
        &mut entry_point,
        vm,
        syscall_handler,
        cheatnet_state,
        remaining_gas,
    )
    .map_err(|error| {
        SyscallExecutionError::from_self_or_revert(error.try_extract_revert().map_original(
            |error| {
                error.as_call_contract_execution_error(
                    class_hash,
                    contract_address,
                    entry_point_selector,
                )
            },
        ))
    })?;
    // endregion

    // Restore the old `tx_context`.
    syscall_handler.base.context.tx_context = old_tx_context;

    Ok(retdata_segment)
}

#[expect(clippy::needless_pass_by_value, clippy::result_large_err)]
pub fn get_block_hash_syscall(
    request: GetBlockHashRequest,
    _vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    _remaining_gas: &mut u64,
) -> SyscallResult<GetBlockHashResponse> {
    let contract_address = syscall_handler.storage_address();
    let block_number = request.block_number.0;

    let block_hash = cheatnet_state.get_block_hash_for_contract(
        contract_address,
        block_number,
        syscall_handler,
    )?;

    Ok(GetBlockHashResponse { block_hash })
}

#[expect(clippy::needless_pass_by_value, clippy::result_large_err)]
pub fn storage_read(
    request: StorageReadRequest,
    _vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    _remaining_gas: &mut u64,
) -> SyscallResult<StorageReadResponse> {
    let original_storage_address = syscall_handler.base.call.storage_address;
    maybe_modify_storage_address(syscall_handler, cheatnet_state)?;

    let value = syscall_handler
        .base
        .storage_read(request.address)
        .inspect_err(|_| {
            // Restore state on error before bubbling up
            syscall_handler.base.call.storage_address = original_storage_address;
        })?;

    // Restore the original storage_address
    syscall_handler.base.call.storage_address = original_storage_address;

    Ok(StorageReadResponse { value })
}

#[expect(clippy::needless_pass_by_value, clippy::result_large_err)]
pub fn storage_write(
    request: StorageWriteRequest,
    _vm: &mut VirtualMachine,
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
    _remaining_gas: &mut u64,
) -> SyscallResult<StorageWriteResponse> {
    let original_storage_address = syscall_handler.base.call.storage_address;
    maybe_modify_storage_address(syscall_handler, cheatnet_state)?;

    syscall_handler
        .base
        .storage_write(request.address, request.value)
        .inspect_err(|_| {
            // Restore state on error before bubbling up
            syscall_handler.base.call.storage_address = original_storage_address;
        })?;

    // Restore the original storage_address
    syscall_handler.base.call.storage_address = original_storage_address;

    Ok(StorageWriteResponse {})
}

// This logic is used to modify the storage address to enable using `contract_state_for_testing`
// inside `interact_with_state` closure cheatcode.
fn maybe_modify_storage_address(
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
) -> Result<(), StateError> {
    let contract_address = syscall_handler.storage_address();
    let test_address =
        TryFromHexStr::try_from_hex_str(TEST_ADDRESS).expect("Failed to parse `TEST_ADDRESS`");

    if contract_address != test_address {
        return Ok(());
    }

    let cheated_data = cheatnet_state.get_cheated_data(contract_address);
    if let Some(actual_address) = cheated_data.contract_address {
        let class_hash = syscall_handler
            .base
            .state
            .get_class_hash_at(actual_address)
            .expect("`get_class_hash_at` should never fail");

        if class_hash == ClassHash::default() {
            return Err(StateError::StateReadError(format!(
                "Failed to interact with contract state because no contract is deployed at address {actual_address}"
            )));
        }

        syscall_handler.base.call.storage_address = actual_address;
    }

    Ok(())
}
