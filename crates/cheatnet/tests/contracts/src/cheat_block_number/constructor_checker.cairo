#[starknet::interface]
trait IConstructorCheatBlockNumberChecker<TContractState> {
    fn get_stored_block_number(ref self: TContractState) -> u64;
    fn get_block_number(ref self: TContractState) -> u64;
}

#[starknet::contract]
mod ConstructorCheatBlockNumberChecker {
    use core::box::BoxTrait;
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        blk_nb: u64,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        let block_numb = starknet::get_block_info().unbox().block_number;
        self.blk_nb.write(block_numb);
    }

    #[abi(embed_v0)]
    impl IConstructorCheatBlockNumberChecker of super::IConstructorCheatBlockNumberChecker<
        ContractState,
    > {
        fn get_stored_block_number(ref self: ContractState) -> u64 {
            self.blk_nb.read()
        }

        fn get_block_number(ref self: ContractState) -> u64 {
            starknet::get_block_info().unbox().block_number
        }
    }
}
