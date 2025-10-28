use crate::runtime_extensions::call_to_blockifier_runtime_extension::execution::cheated_syscalls::execute_deployment;
use crate::runtime_extensions::native::native_syscall_handler::BaseSyscallResult;
use crate::state::CheatnetState;
use blockifier::execution::call_info::CallInfo;
use blockifier::execution::entry_point::ConstructorContext;
use blockifier::execution::syscalls::syscall_base::SyscallHandlerBase;
use blockifier::execution::syscalls::vm_syscall_utils::SyscallSelector;
use starknet_api::core::{ClassHash, ContractAddress, calculate_contract_address};
use starknet_api::transaction::fields::{Calldata, ContractAddressSalt};

#[expect(clippy::match_bool)]
// Copied from blockifer/src/execution/syscalls/syscall_base.rs
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

    // region: Modified blockifer code
    // removed code
    // endregion

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
    // region: Modified blockifer code
    let call_info = execute_deployment(
        syscall_handler_base.state,
        cheatnet_state,
        syscall_handler_base.context,
        &ctor_context,
        constructor_calldata,
        remaining_gas,
    )?;
    // endregion
    Ok((deployed_contract_address, call_info))
}
