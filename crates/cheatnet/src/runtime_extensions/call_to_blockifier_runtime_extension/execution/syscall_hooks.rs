use crate::runtime_extensions::forge_runtime_extension::cheatcodes::spy_messages_to_l1::MessageToL1;
use crate::{
    runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::Event,
    state::CheatnetState,
};
use blockifier::execution::call_info::OrderedL2ToL1Message;
use blockifier::execution::{
    call_info::OrderedEvent, deprecated_syscalls::hint_processor::DeprecatedSyscallHintProcessor,
    syscalls::hint_processor::SyscallHintProcessor,
};
use starknet_api::core::ContractAddress;

pub trait SyscallHintProcessorExt {
    fn contract_address(&self) -> ContractAddress;
    fn last_event(&self) -> &OrderedEvent;
    fn last_l2_to_l1_message(&self) -> &OrderedL2ToL1Message;
}

impl SyscallHintProcessorExt for SyscallHintProcessor<'_> {
    fn contract_address(&self) -> ContractAddress {
        self.base
            .call
            .code_address
            .unwrap_or(self.base.call.storage_address)
    }
    fn last_event(&self) -> &OrderedEvent {
        self.base.events.last().unwrap()
    }
    fn last_l2_to_l1_message(&self) -> &OrderedL2ToL1Message {
        self.base.l2_to_l1_messages.last().unwrap()
    }
}

impl SyscallHintProcessorExt for DeprecatedSyscallHintProcessor<'_> {
    fn contract_address(&self) -> ContractAddress {
        self.storage_address
    }
    fn last_event(&self) -> &OrderedEvent {
        self.events.last().unwrap()
    }

    fn last_l2_to_l1_message(&self) -> &OrderedL2ToL1Message {
        self.l2_to_l1_messages.last().unwrap()
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

pub fn send_message_to_l1_syscall_hook(
    syscall_handler: &impl SyscallHintProcessorExt,
    cheatnet_state: &mut CheatnetState,
) {
    let contract_address = syscall_handler.contract_address();
    let last_message = syscall_handler.last_l2_to_l1_message();

    cheatnet_state
        .detected_messages_to_l1
        .push(MessageToL1::from_ordered_message(
            last_message,
            contract_address,
        ));
}
