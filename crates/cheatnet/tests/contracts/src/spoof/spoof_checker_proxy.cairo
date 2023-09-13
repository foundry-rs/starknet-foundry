use starknet::ContractAddress;

#[starknet::interface]
trait ISpoofChecker<TContractState> {
    fn get_tx_hash(ref self: TContractState) -> felt252;
}


#[starknet::interface]
trait ISpoofCheckerProxy<TContractState> {
    fn get_spoof_checkers_tx_hash(ref self: TContractState, address: ContractAddress) -> felt252;
}

#[starknet::contract]
mod SpoofCheckerProxy {
    use starknet::ContractAddress;
    use super::ISpoofCheckerDispatcherTrait;
    use super::ISpoofCheckerDispatcher;

    #[storage]
    struct Storage {}

    #[external(v0)]
    impl ISpoofCheckerProxy of super::ISpoofCheckerProxy<ContractState> {
        fn get_spoof_checkers_tx_hash(ref self: ContractState, address: ContractAddress) -> felt252 {
            let spoof_checker = ISpoofCheckerDispatcher { contract_address: address };
            spoof_checker.get_tx_hash()
        }
    }
}
