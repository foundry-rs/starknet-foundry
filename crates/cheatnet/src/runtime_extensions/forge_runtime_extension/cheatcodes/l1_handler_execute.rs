use crate::{
    runtime_extensions::call_to_blockifier_runtime_extension::rpc::{call_l1_handler, CallResult},
    state::CheatnetState,
};
use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;
use cairo_felt::Felt252;
use starknet_api::core::ContractAddress;

pub fn l1_handler_execute(
    syscall_handler: &mut SyscallHintProcessor,
    cheatnet_state: &mut CheatnetState,
    contract_address: ContractAddress,
    function_selector: &Felt252,
    from_address: &Felt252,
    payload: &[Felt252],
) -> CallResult {
    let mut calldata = vec![from_address.clone()];
    calldata.extend_from_slice(payload);

    call_l1_handler(
        syscall_handler,
        cheatnet_state,
        &contract_address,
        function_selector,
        calldata.as_slice(),
    )
}
