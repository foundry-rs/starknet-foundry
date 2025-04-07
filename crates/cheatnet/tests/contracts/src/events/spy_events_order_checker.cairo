use starknet::ContractAddress;

#[starknet::interface]
trait ISpyEventsOrderChecker<TContractState> {
    fn emit_and_call_another(
        ref self: TContractState,
        first_data: felt252,
        second_data: felt252,
        third_data: felt252,
        another_contract_address: ContractAddress,
    );
}

#[starknet::contract]
mod SpyEventsOrderChecker {
    use starknet::ContractAddress;

    #[starknet::interface]
    trait ISpyEventsChecker<TContractState> {
        fn emit_one_event(ref self: TContractState, some_data: felt252);
    }

    #[storage]
    struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        SecondEvent: SecondEvent,
        ThirdEvent: ThirdEvent,
    }

    #[derive(Drop, starknet::Event)]
    struct SecondEvent {
        data: felt252,
    }

    #[derive(Drop, starknet::Event)]
    struct ThirdEvent {
        data: felt252,
    }

    #[abi(embed_v0)]
    impl ISpyEventsOrderCheckerImpl of super::ISpyEventsOrderChecker<ContractState> {
        fn emit_and_call_another(
            ref self: ContractState,
            first_data: felt252,
            second_data: felt252,
            third_data: felt252,
            another_contract_address: ContractAddress,
        ) {
            self.emit(Event::SecondEvent(SecondEvent { data: first_data }));

            let spy_events_checker = ISpyEventsCheckerDispatcher {
                contract_address: another_contract_address,
            };
            spy_events_checker.emit_one_event(second_data);

            self.emit(Event::ThirdEvent(ThirdEvent { data: third_data }));
        }
    }
}
