use crate::{
    runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::Event,
    state::CheatnetState,
};
use blockifier::execution::{
    call_info::OrderedEvent, deprecated_syscalls::hint_processor::DeprecatedSyscallHintProcessor,
    syscalls::hint_processor::SyscallHintProcessor,
};
use starknet_api::core::ContractAddress;

pub trait SyscallHintProcessorExt {
    fn contract_address(&self) -> ContractAddress;
    fn last_event(&self) -> &OrderedEvent;
}

impl SyscallHintProcessorExt for SyscallHintProcessor<'_> {
    fn contract_address(&self) -> ContractAddress {
        self.call.code_address.unwrap_or(self.call.storage_address)
    }
    fn last_event(&self) -> &OrderedEvent {
        self.events.last().unwrap()
    }
}

impl SyscallHintProcessorExt for DeprecatedSyscallHintProcessor<'_> {
    fn contract_address(&self) -> ContractAddress {
        self.storage_address
    }
    fn last_event(&self) -> &OrderedEvent {
        self.events.last().unwrap()
    }
}

pub fn emit_event_hook(
    syscall_handler: &impl SyscallHintProcessorExt,
    cheatnet_state: &mut CheatnetState,
) {
    let contract_address = syscall_handler.contract_address();
    let last_event = syscall_handler.last_event();
    cheatnet_state
        .detected_events
        .push(Event::from_ordered_event(last_event, contract_address));
}
