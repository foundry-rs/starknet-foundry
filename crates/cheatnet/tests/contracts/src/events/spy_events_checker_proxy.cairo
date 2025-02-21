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
}

#[starknet::contract]
mod SpyEventsCheckerProxy {
    use starknet::ContractAddress;
    use super::ISpyEventsCheckerDispatcherTrait;
    use super::ISpyEventsCheckerDispatcher;
    use core::starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        proxied_address: ContractAddress,
    }

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

    #[constructor]
    fn constructor(ref self: ContractState, proxied_address: ContractAddress) {
        self.proxied_address.write(proxied_address);
    }

    #[abi(embed_v0)]
    impl ISpyEventsChecker of super::ISpyEventsChecker<ContractState> {
        fn do_not_emit(ref self: ContractState) {
            let spy_events_checker = ISpyEventsCheckerDispatcher {
                contract_address: self.proxied_address.read()
            };
            spy_events_checker.do_not_emit();
        }

        fn emit_one_event(ref self: ContractState, some_data: felt252) {
            self.emit(Event::FirstEvent(FirstEvent { some_data }));

            let spy_events_checker = ISpyEventsCheckerDispatcher {
                contract_address: self.proxied_address.read()
            };
            spy_events_checker.emit_one_event(some_data);
        }

        fn emit_two_events(
            ref self: ContractState, some_data: felt252, some_more_data: ContractAddress
        ) {
            self.emit(Event::FirstEvent(FirstEvent { some_data }));
            self.emit(Event::SecondEvent(SecondEvent { some_data, some_more_data }));

            let spy_events_checker = ISpyEventsCheckerDispatcher {
                contract_address: self.proxied_address.read()
            };
            spy_events_checker.emit_two_events(some_data, some_more_data);
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

            let spy_events_checker = ISpyEventsCheckerDispatcher {
                contract_address: self.proxied_address.read()
            };
            spy_events_checker.emit_three_events(some_data, some_more_data, even_more_data);
        }
    }
}
