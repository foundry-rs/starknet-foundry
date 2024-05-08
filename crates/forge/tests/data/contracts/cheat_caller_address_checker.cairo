#[starknet::interface]
trait ICheatCallerAddressChecker<TContractState> {
    fn get_caller_address(ref self: TContractState) -> felt252;
    fn get_caller_address_and_emit_event(ref self: TContractState) -> felt252;
}

#[starknet::contract]
mod CheatCallerAddressChecker {
    use box::BoxTrait;
    use starknet::ContractAddressIntoFelt252;
    use starknet::ContractAddress;
    use option::Option;
    use traits::Into;

    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        CallerAddressEmitted: CallerAddressEmitted
    }

    #[derive(Drop, starknet::Event)]
    struct CallerAddressEmitted {
        caller_address: felt252
    }

    #[abi(embed_v0)]
    impl ICheatCallerAddressChecker of super::ICheatCallerAddressChecker<ContractState> {
        fn get_caller_address(ref self: ContractState) -> felt252 {
            starknet::get_caller_address().into()
        }

        fn get_caller_address_and_emit_event(ref self: ContractState) -> felt252 {
            let caller_address = starknet::get_caller_address().into();
            self.emit(Event::CallerAddressEmitted(CallerAddressEmitted { caller_address }));
            caller_address
        }
    }
}
