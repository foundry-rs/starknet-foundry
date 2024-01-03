use starknet::ContractAddress;

#[starknet::interface]
trait ISpoofChecker<TContractState> {
    fn get_transaction_hash(self: @TContractState) -> felt252;
}


#[starknet::interface]
trait ISpoofCheckerProxy<TContractState> {
    fn get_spoof_checkers_tx_hash(self: @TContractState, address: ContractAddress) -> felt252;
}

#[starknet::contract]
mod SpoofCheckerProxy {
    use starknet::ContractAddress;
    use super::ISpoofCheckerDispatcherTrait;
    use super::ISpoofCheckerDispatcher;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ISpoofCheckerProxy of super::ISpoofCheckerProxy<ContractState> {
        fn get_spoof_checkers_tx_hash(self: @ContractState, address: ContractAddress) -> felt252 {
            let spoof_checker = ISpoofCheckerDispatcher { contract_address: address };
            spoof_checker.get_transaction_hash()
        }
    }
}
