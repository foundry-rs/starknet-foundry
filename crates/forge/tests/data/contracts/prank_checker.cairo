#[starknet::interface]
trait IPrankChecker<TContractState> {
    fn get_caller_address(ref self: TContractState) -> felt252;
    fn get_caller_address_and_emit_event(ref self: TContractState) -> felt252;
}

#[starknet::contract]
mod PrankChecker {
    use box::BoxTrait;
    use starknet::ContractAddressIntoFelt252;
    use starknet::ContractAddress;
    use option::Option;
    use traits::Into;
    use array::ArrayTrait;

    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        NewEventCheck: NewEventCheck,
        CallerAddressEmitted: CallerAddressEmitted,
    }

    #[derive(Drop, starknet::Event)]
    struct NewEventCheck {
    }

    #[derive(Drop, starknet::Event)]
    struct CallerAddressEmitted {
        caller_address: felt252
    }

    #[external(v0)]
    impl IPrankChecker of super::IPrankChecker<ContractState> {
        fn get_caller_address(ref self: ContractState) -> felt252 {
            starknet::get_caller_address().into()
        }

        fn get_caller_address_and_emit_event(ref self: ContractState) -> felt252 {
            let caller_address = starknet::get_caller_address().into();
            self.emit(Event::NewEventCheck(NewEventCheck { }));
            caller_address
        }
    }
}
