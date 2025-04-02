#[starknet::interface]
trait ITxHashChecker<TContractState> {
    fn get_stored_tx_hash(self: @TContractState) -> felt252;
    fn get_transaction_hash(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod TxHashChecker {
    use core::box::BoxTrait;
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        stored_tx_hash: felt252,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        let tx_hash = starknet::get_tx_info().unbox().transaction_hash;
        self.stored_tx_hash.write(tx_hash);
    }

    #[abi(embed_v0)]
    impl ITxHashChecker of super::ITxHashChecker<ContractState> {
        fn get_stored_tx_hash(self: @ContractState) -> felt252 {
            self.stored_tx_hash.read()
        }

        fn get_transaction_hash(self: @ContractState) -> felt252 {
            starknet::get_tx_info().unbox().transaction_hash
        }
    }
}
