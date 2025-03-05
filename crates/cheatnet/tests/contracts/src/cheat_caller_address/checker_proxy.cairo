use starknet::ContractAddress;

#[starknet::interface]
trait ICheatCallerAddressChecker<TContractState> {
    fn get_caller_address(self: @TContractState) -> felt252;
    fn get_caller_address_and_emit_event(self: @TContractState) -> felt252;
}

#[starknet::interface]
trait ICheatCallerAddressCheckerProxy<TContractState> {
    fn get_cheated_caller_address(self: @TContractState, address: ContractAddress) -> felt252;
    fn get_caller_address(self: @TContractState) -> felt252;
    fn call_proxy(self: @TContractState, address: ContractAddress) -> (felt252, felt252);
}

#[starknet::contract]
mod CheatCallerAddressCheckerProxy {
    use starknet::ContractAddress;
    use super::ICheatCallerAddressCheckerDispatcherTrait;
    use super::ICheatCallerAddressCheckerDispatcher;
    use super::ICheatCallerAddressCheckerProxyDispatcherTrait;
    use super::ICheatCallerAddressCheckerProxyDispatcher;
    use starknet::{get_contract_address, get_caller_address};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatCallerAddressCheckerProxy of super::ICheatCallerAddressCheckerProxy<ContractState> {
        fn get_cheated_caller_address(self: @ContractState, address: ContractAddress) -> felt252 {
            let cheat_caller_address_checker = ICheatCallerAddressCheckerDispatcher {
                contract_address: address,
            };
            cheat_caller_address_checker.get_caller_address()
        }

        fn get_caller_address(self: @ContractState) -> felt252 {
            starknet::get_caller_address().into()
        }

        fn call_proxy(self: @ContractState, address: ContractAddress) -> (felt252, felt252) {
            let dispatcher = ICheatCallerAddressCheckerProxyDispatcher {
                contract_address: address,
            };
            let caller_address: felt252 = get_caller_address().into();
            let res = dispatcher.get_cheated_caller_address(get_contract_address());
            (caller_address, res)
        }
    }
}
