use starknet::ContractAddress;

#[starknet::interface]
trait ISpoofChecker<TContractState> {
    fn get_transaction_hash(self: @TContractState) -> felt252;
}


#[starknet::interface]
trait ISpoofCheckerProxy<TContractState> {
    fn get_spoof_checkers_tx_hash(self: @TContractState, address: ContractAddress) -> felt252;
    fn get_transaction_hash(self: @TContractState) -> felt252;
    fn call_proxy(self: @TContractState, address: ContractAddress) -> (felt252, felt252);
}

#[starknet::contract]
mod SpoofCheckerProxy {
    use starknet::ContractAddress;
    use super::ISpoofCheckerDispatcherTrait;
    use super::ISpoofCheckerDispatcher;
    use super::ISpoofCheckerProxyDispatcherTrait;
    use super::ISpoofCheckerProxyDispatcher;
    use starknet::get_contract_address;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ISpoofCheckerProxy of super::ISpoofCheckerProxy<ContractState> {
        fn get_spoof_checkers_tx_hash(self: @ContractState, address: ContractAddress) -> felt252 {
            let spoof_checker = ISpoofCheckerDispatcher { contract_address: address };
            spoof_checker.get_transaction_hash()
        }

        fn get_transaction_hash(self: @ContractState) -> felt252 {
            starknet::get_tx_info().unbox().transaction_hash
        }

        fn call_proxy(
            self: @ContractState, address: ContractAddress
        ) -> (felt252, felt252) {
            let dispatcher = ISpoofCheckerProxyDispatcher { contract_address: address };
            let tx_hash = self.get_transaction_hash();
            let res = dispatcher.get_spoof_checkers_tx_hash(get_contract_address());
            (tx_hash, res)
        }
    }
}
