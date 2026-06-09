#[starknet::interface]
trait IEventsChecker<TContractState> {
    fn emit_event(ref self: TContractState, value: felt252);
    fn emit_two_events(ref self: TContractState, first_value: felt252, second_value: felt252);
    fn do_not_emit(self: @TContractState);
    fn emit_custom_event(ref self: TContractState, key: felt252, value: felt252);
}

#[starknet::contract]
mod EventsChecker {
    use starknet::SyscallResultTrait;

    #[storage]
    struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        ValueEmitted: ValueEmitted,
    }

    #[derive(Drop, starknet::Event)]
    struct ValueEmitted {
        value: felt252,
    }

    #[abi(embed_v0)]
    impl IEventsCheckerImpl of super::IEventsChecker<ContractState> {
        fn emit_event(ref self: ContractState, value: felt252) {
            self.emit(Event::ValueEmitted(ValueEmitted { value }));
        }

        fn emit_two_events(ref self: ContractState, first_value: felt252, second_value: felt252) {
            self.emit(Event::ValueEmitted(ValueEmitted { value: first_value }));
            self.emit(Event::ValueEmitted(ValueEmitted { value: second_value }));
        }

        fn do_not_emit(self: @ContractState) {}

        fn emit_custom_event(ref self: ContractState, key: felt252, value: felt252) {
            starknet::emit_event_syscall(array![key].span(), array![value].span()).unwrap_syscall();
        }
    }
}
