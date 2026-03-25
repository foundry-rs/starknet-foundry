use starknet::ClassHash;

#[starknet::interface]
trait ICheatCallerAddressChecker<TContractState> {
    fn get_caller_address(ref self: TContractState) -> felt252;
    fn get_caller_address_and_emit_event(ref self: TContractState) -> felt252;
}

#[starknet::interface]
trait ICheatCallerAddressLibraryCallChecker<TContractState> {
    fn get_caller_address_via_library_call(
        self: @TContractState, class_hash: ClassHash,
    ) -> felt252;
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

#[starknet::contract]
mod CheatCallerAddressLibraryCallChecker {
    use starknet::ClassHash;
    use super::ICheatCallerAddressCheckerDispatcherTrait;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl CheatCallerAddressLibraryCallCheckerImpl of super::ICheatCallerAddressLibraryCallChecker<ContractState> {
        fn get_caller_address_via_library_call(
            self: @ContractState, class_hash: ClassHash,
        ) -> felt252 {
            super::ICheatCallerAddressCheckerLibraryDispatcher { class_hash }.get_caller_address()
        }
    }
}
