use crate::{
    runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::Event,
    state::CheatnetState,
};
use blockifier::execution::{
    call_info::OrderedEvent, deprecated_syscalls::hint_processor::DeprecatedSyscallHintProcessor,
    syscalls::hint_processor::SyscallHintProcessor,
};
use starknet_api::core::ContractAddress;

pub fn emit_event_hook(
    syscall_handler: &SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
) {
    let contract_address = syscall_handler
        .call
        .code_address
        .unwrap_or(syscall_handler.call.storage_address);

    emit_event_hook_inner(
        contract_address,
        syscall_handler.events.last().unwrap(),
        cheatnet_state,
    );
}
pub fn emit_event_hook_deprecated(
    syscall_handler: &DeprecatedSyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
) {
    let contract_address = syscall_handler.storage_address;

    emit_event_hook_inner(
        contract_address,
        syscall_handler.events.last().unwrap(),
        cheatnet_state,
    );
}

pub fn emit_event_hook_inner(
    contract_address: ContractAddress,
    ordered_event: &OrderedEvent,
    cheatnet_state: &mut CheatnetState,
) {
    for spy_on in &cheatnet_state.spies {
        if spy_on.does_spy(contract_address) {
            let event = Event::from_ordered_event(ordered_event, contract_address);
            cheatnet_state.detected_events.push(event);
            break;
        }
    }
}
