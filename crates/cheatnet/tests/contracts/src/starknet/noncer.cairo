#[starknet::interface]
trait INoncer<TContractState> {
    fn write_nonce(ref self: TContractState);
    fn read_nonce(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod Noncer {
    use core::array::ArrayTrait;
    use core::box::BoxTrait;
    use starknet::get_tx_info;
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        nonce: felt252,
    }

    #[abi(embed_v0)]
    impl INoncerImpl of super::INoncer<ContractState> {
        fn write_nonce(ref self: ContractState) {
            let tx_info = get_tx_info().unbox();
            let nonce = tx_info.nonce;
            self.nonce.write(nonce);
        }

        fn read_nonce(self: @ContractState) -> felt252 {
            self.nonce.read()
        }
    }
}
