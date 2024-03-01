use starknet::ContractAddress;

#[starknet::interface]
trait IPrankChecker<TContractState> {
    fn get_caller_address(self: @TContractState) -> felt252;
    fn get_caller_address_and_emit_event(self: @TContractState) -> felt252;
}

#[starknet::interface]
trait IPrankCheckerProxy<TContractState> {
    fn get_prank_checkers_caller_address(
        self: @TContractState, address: ContractAddress
    ) -> felt252;
    fn get_caller_address(self: @TContractState) -> felt252;
    fn call_proxy(self: @TContractState, address: ContractAddress) -> (felt252, felt252);
}

#[starknet::contract]
mod PrankCheckerProxy {
    use starknet::ContractAddress;
    use super::IPrankCheckerDispatcherTrait;
    use super::IPrankCheckerDispatcher;
    use super::IPrankCheckerProxyDispatcherTrait;
    use super::IPrankCheckerProxyDispatcher;
    use starknet::{get_contract_address, get_caller_address};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IPrankCheckerProxy of super::IPrankCheckerProxy<ContractState> {
        fn get_prank_checkers_caller_address(
            self: @ContractState, address: ContractAddress
        ) -> felt252 {
            let prank_checker = IPrankCheckerDispatcher { contract_address: address };
            prank_checker.get_caller_address()
        }

        fn get_caller_address(self: @ContractState) -> felt252 {
            starknet::get_caller_address().into()
        }

        fn call_proxy(self: @ContractState, address: ContractAddress) -> (felt252, felt252) {
            let dispatcher = IPrankCheckerProxyDispatcher { contract_address: address };
            let caller_address: felt252 = get_caller_address().into();
            let res = dispatcher.get_prank_checkers_caller_address(get_contract_address());
            (caller_address, res)
        }
    }
}
