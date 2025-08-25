use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::entry_point::execute_call_entry_point;
use crate::runtime_extensions::native::native_syscall_handler::BaseSyscallResult;
use crate::state::CheatnetState;
use blockifier::execution::call_info::CallInfo;
use blockifier::execution::entry_point::{
    CallEntryPoint, CallType, ConstructorContext, ConstructorEntryPointExecutionResult,
    EntryPointExecutionContext, handle_empty_constructor,
};
use blockifier::execution::errors::ConstructorEntryPointExecutionError;
use blockifier::execution::syscalls::syscall_base::SyscallHandlerBase;
use blockifier::execution::syscalls::vm_syscall_utils::SyscallSelector;
use blockifier::state::errors::StateError;
use blockifier::state::state_api::State;
use starknet_api::contract_class::EntryPointType;
use starknet_api::core::{ClassHash, ContractAddress, calculate_contract_address};
use starknet_api::transaction::fields::{Calldata, ContractAddressSalt};

#[allow(clippy::result_large_err)]
pub fn execute_constructor_entry_point(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    context: &mut EntryPointExecutionContext,
    ctor_context: ConstructorContext,
    calldata: Calldata,
    remaining_gas: &mut u64,
) -> ConstructorEntryPointExecutionResult<CallInfo> {
    // Ensure the class is declared (by reading it).
    let compiled_class = state
        .get_compiled_class(ctor_context.class_hash)
        .map_err(|error| {
            ConstructorEntryPointExecutionError::new(error.into(), &ctor_context, None)
        })?;
    let Some(constructor_selector) = compiled_class.constructor_selector() else {
        // Contract has no constructor.
        return handle_empty_constructor(
            compiled_class,
            context,
            &ctor_context,
            calldata,
            *remaining_gas,
        )
        .map_err(|error| ConstructorEntryPointExecutionError::new(error, &ctor_context, None));
    };

    let mut constructor_call = CallEntryPoint {
        class_hash: None,
        code_address: ctor_context.code_address,
        entry_point_type: EntryPointType::Constructor,
        entry_point_selector: constructor_selector,
        calldata,
        storage_address: ctor_context.storage_address,
        caller_address: ctor_context.caller_address,
        call_type: CallType::Call,
        initial_gas: *remaining_gas,
    };

    let call_info =
        execute_call_entry_point(&mut constructor_call, state, cheatnet_state, context, false)
            .map_err(|error| {
                ConstructorEntryPointExecutionError::new(
                    error,
                    &ctor_context,
                    Some(constructor_selector),
                )
            })?;

    Ok(call_info)
}

fn execute_deployment(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    context: &mut EntryPointExecutionContext,
    ctor_context: ConstructorContext,
    constructor_calldata: Calldata,
    remaining_gas: &mut u64,
) -> ConstructorEntryPointExecutionResult<CallInfo> {
    // Address allocation in the state is done before calling the constructor, so that it is
    // visible from it.
    let deployed_contract_address = ctor_context.storage_address;
    let current_class_hash =
        state
            .get_class_hash_at(deployed_contract_address)
            .map_err(|error| {
                ConstructorEntryPointExecutionError::new(error.into(), &ctor_context, None)
            })?;
    if current_class_hash != ClassHash::default() {
        return Err(ConstructorEntryPointExecutionError::new(
            StateError::UnavailableContractAddress(deployed_contract_address).into(),
            &ctor_context,
            None,
        ));
    }

    state
        .set_class_hash_at(deployed_contract_address, ctor_context.class_hash)
        .map_err(|error| {
            ConstructorEntryPointExecutionError::new(error.into(), &ctor_context, None)
        })?;

    execute_constructor_entry_point(
        state,
        cheatnet_state,
        context,
        ctor_context,
        constructor_calldata,
        remaining_gas,
    )
}

pub fn deploy(
    syscall_handler_base: &mut SyscallHandlerBase,
    cheatnet_state: &mut CheatnetState,
    class_hash: ClassHash,
    contract_address_salt: ContractAddressSalt,
    constructor_calldata: Calldata,
    deploy_from_zero: bool,
    remaining_gas: &mut u64,
) -> BaseSyscallResult<(ContractAddress, CallInfo)> {
    syscall_handler_base
        .increment_syscall_linear_factor_by(&SyscallSelector::Deploy, constructor_calldata.0.len());
    // let versioned_constants = &syscall_handler_base
    //     .context
    //     .tx_context
    //     .block_context
    //     .versioned_constants;
    // TODO support for reject
    // if should_reject_deploy(
    //     versioned_constants.disable_deploy_in_validation_mode,
    //     syscall_handler_base.context.execution_mode,
    // ) {
    //     syscall_handler_base.reject_syscall_in_validate_mode("deploy")?;
    // }

    let deployer_address = syscall_handler_base.call.storage_address;
    let deployer_address_for_calculation = match deploy_from_zero {
        true => ContractAddress::default(),
        false => deployer_address,
    };
    let deployed_contract_address = calculate_contract_address(
        contract_address_salt,
        class_hash,
        &constructor_calldata,
        deployer_address_for_calculation,
    )?;

    let ctor_context = ConstructorContext {
        class_hash,
        code_address: Some(deployed_contract_address),
        storage_address: deployed_contract_address,
        caller_address: deployer_address,
    };
    let call_info = execute_deployment(
        syscall_handler_base.state,
        cheatnet_state,
        syscall_handler_base.context,
        ctor_context,
        constructor_calldata,
        remaining_gas,
    )?;
    Ok((deployed_contract_address, call_info))
}
