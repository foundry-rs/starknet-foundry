use starknet::ContractAddress;

#[starknet::interface]
trait IPrankChecker<TContractState> {
    fn get_caller_address(ref self: TContractState) -> felt252;
    fn get_caller_address_and_emit_event(ref self: TContractState) -> felt252;
}


#[starknet::interface]
trait IPrankCheckerProxy<TContractState> {
    fn get_prank_checkers_caller_address(
        ref self: TContractState, address: ContractAddress
    ) -> felt252;
}

#[starknet::contract]
mod PrankCheckerProxy {
    use starknet::ContractAddress;
    use super::IPrankCheckerDispatcherTrait;
    use super::IPrankCheckerDispatcher;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IPrankCheckerProxy of super::IPrankCheckerProxy<ContractState> {
        fn get_prank_checkers_caller_address(
            ref self: ContractState, address: ContractAddress
        ) -> felt252 {
            let prank_checker = IPrankCheckerDispatcher { contract_address: address };
            prank_checker.get_caller_address()
        }
    }
}
