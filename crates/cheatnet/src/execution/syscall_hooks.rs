use crate::cheatcodes::spy_events::Event;
use crate::execution::cheatable_syscall_handler::CheatableSyscallHandler;

pub fn emit_event_hook(syscall_handler: &mut CheatableSyscallHandler<'_>) {
    let contract_address = syscall_handler
        .child
        .call
        .code_address
        .unwrap_or(syscall_handler.child.call.storage_address);

    for spy_on in &mut syscall_handler.cheatnet_state.spies {
        if spy_on.does_spy(contract_address) {
            let event = Event::from_ordered_event(
                syscall_handler.child.events.last().unwrap(),
                contract_address,
            );
            syscall_handler.cheatnet_state.detected_events.push(event);
            break;
        }
    }
}
