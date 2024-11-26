#[starknet::interface]
trait IConstructorCheatBlockHashChecker<TContractState> {
    fn get_stored_block_hash(ref self: TContractState) -> felt252;
    fn get_block_hash(ref self: TContractState) -> felt252;
}

#[starknet::contract]
mod ConstructorCheatBlockHashChecker {
    use box::BoxTrait;
    use starknet::{get_block_info, get_block_hash_syscall};

    #[storage]
    struct Storage {
        blk_hash: felt252,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        let block_info = get_block_info().unbox();
        let blk_hash = get_block_hash_syscall(block_info.block_number - 10).unwrap_syscall();
        self.blk_hash.write(blk_hash);
    }

    #[abi(embed_v0)]
    impl IConstructorCheatBlockHashChecker of super::IConstructorCheatBlockHashChecker<
        ContractState
    > {
        fn get_stored_block_hash(ref self: ContractState) -> felt252 {
            self.blk_hash.read()
        }

        fn get_block_hash(ref self: ContractState) -> felt252 {
            let block_info = get_block_info().unbox();
            let block_hash = get_block_hash_syscall(block_info.block_number - 10).unwrap_syscall();

            block_hash
        }
    }
}
