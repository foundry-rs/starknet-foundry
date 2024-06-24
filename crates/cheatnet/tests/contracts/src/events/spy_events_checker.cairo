use starknet::ContractAddress;

#[starknet::interface]
trait ISpyEventsChecker<TContractState> {
    fn do_not_emit(ref self: TContractState);
    fn emit_one_event(ref self: TContractState, some_data: felt252);
    fn emit_two_events(
        ref self: TContractState, some_data: felt252, some_more_data: ContractAddress
    );
    fn emit_three_events(
        ref self: TContractState,
        some_data: felt252,
        some_more_data: ContractAddress,
        even_more_data: u256
    );
    fn emit_event_syscall(ref self: TContractState, some_key: felt252, some_data: felt252);
}

#[starknet::contract]
mod SpyEventsChecker {
    use starknet::ContractAddress;
    use starknet::SyscallResultTrait;

    #[storage]
    struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        FirstEvent: FirstEvent,
        SecondEvent: SecondEvent,
        ThirdEvent: ThirdEvent,
    }

    #[derive(Drop, starknet::Event)]
    struct FirstEvent {
        some_data: felt252
    }

    #[derive(Drop, starknet::Event)]
    struct SecondEvent {
        some_data: felt252,
        #[key]
        some_more_data: ContractAddress
    }

    #[derive(Drop, starknet::Event)]
    struct ThirdEvent {
        some_data: felt252,
        some_more_data: ContractAddress,
        even_more_data: u256
    }

    #[abi(embed_v0)]
    impl ISpyEventsChecker of super::ISpyEventsChecker<ContractState> {
        fn do_not_emit(ref self: ContractState) {}

        fn emit_one_event(ref self: ContractState, some_data: felt252) {
            self.emit(Event::FirstEvent(FirstEvent { some_data }));
        }

        fn emit_two_events(
            ref self: ContractState, some_data: felt252, some_more_data: ContractAddress
        ) {
            self.emit(Event::FirstEvent(FirstEvent { some_data }));
            self.emit(Event::SecondEvent(SecondEvent { some_data, some_more_data }));
        }

        fn emit_three_events(
            ref self: ContractState,
            some_data: felt252,
            some_more_data: ContractAddress,
            even_more_data: u256
        ) {
            self.emit(Event::FirstEvent(FirstEvent { some_data }));
            self.emit(Event::SecondEvent(SecondEvent { some_data, some_more_data }));
            self.emit(Event::ThirdEvent(ThirdEvent { some_data, some_more_data, even_more_data }));
        }

        fn emit_event_syscall(ref self: ContractState, some_key: felt252, some_data: felt252) {
            starknet::emit_event_syscall(array![some_key].span(), array![some_data].span())
                .unwrap_syscall();
        }
    }
}
