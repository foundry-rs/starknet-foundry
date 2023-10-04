use starknet::ContractAddress;

#[starknet::interface]
trait BlockHashCheckerInterface<TContractState> {
    fn write_block(ref self: TContractState);
    fn read_block_hash(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod BlockHashChecker {
    use core::starknet::SyscallResultTrait;
    use array::ArrayTrait;
    use starknet::{get_block_info, get_block_hash_syscall};
    use box::BoxTrait;
    use starknet::ContractAddress;

    #[storage]
    struct Storage {
        block_hash: felt252,
    }

    #[external(v0)]
    impl BlockHashCheckerImpl of super::BlockHashCheckerInterface<ContractState> {
        fn write_block(ref self: ContractState) {
            let block_info = get_block_info().unbox();

            let block_hash = get_block_hash_syscall(block_info.block_number - 10).unwrap_syscall();
            self.block_hash.write(block_hash);
        }

        fn read_block_hash(self: @ContractState) -> felt252 {
            self.block_hash.read()
        }
    }
}
