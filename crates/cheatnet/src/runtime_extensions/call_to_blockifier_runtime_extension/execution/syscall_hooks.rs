use blockifier::execution::syscalls::hint_processor::SyscallHintProcessor;

use crate::{
    runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::Event,
    state::CheatnetState,
};

pub fn emit_event_hook(
    syscall_handler: &mut SyscallHintProcessor<'_>,
    cheatnet_state: &mut CheatnetState,
) {
    let contract_address = syscall_handler
        .call
        .code_address
        .unwrap_or(syscall_handler.call.storage_address);

    for spy_on in &mut cheatnet_state.spies {
        if spy_on.does_spy(contract_address) {
            let event =
                Event::from_ordered_event(syscall_handler.events.last().unwrap(), contract_address);
            cheatnet_state.detected_events.push(event);
            break;
        }
    }
}
