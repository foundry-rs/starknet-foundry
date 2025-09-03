use starknet::ContractAddress;

#[starknet::interface]
trait ICheatTxInfoChecker<TContractState> {
    fn get_transaction_hash(self: @TContractState) -> felt252;
}


#[starknet::interface]
trait ITxHashCheckerProxy<TContractState> {
    fn get_checkers_tx_hash(self: @TContractState, address: ContractAddress) -> felt252;
    fn get_transaction_hash(self: @TContractState) -> felt252;
    fn call_proxy(self: @TContractState, address: ContractAddress) -> (felt252, felt252);
}

#[starknet::contract]
mod TxHashCheckerProxy {
    use starknet::{ContractAddress, get_contract_address};
    use super::{
        ICheatTxInfoCheckerDispatcher, ICheatTxInfoCheckerDispatcherTrait,
        ITxHashCheckerProxyDispatcher, ITxHashCheckerProxyDispatcherTrait,
    };

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ITxHashCheckerProxy of super::ITxHashCheckerProxy<ContractState> {
        fn get_checkers_tx_hash(self: @ContractState, address: ContractAddress) -> felt252 {
            let tx_info_checker = ICheatTxInfoCheckerDispatcher { contract_address: address };
            tx_info_checker.get_transaction_hash()
        }

        fn get_transaction_hash(self: @ContractState) -> felt252 {
            starknet::get_tx_info().unbox().transaction_hash
        }

        fn call_proxy(self: @ContractState, address: ContractAddress) -> (felt252, felt252) {
            let dispatcher = ITxHashCheckerProxyDispatcher { contract_address: address };
            let tx_hash = self.get_transaction_hash();
            let res = dispatcher.get_checkers_tx_hash(get_contract_address());
            (tx_hash, res)
        }
    }
}
