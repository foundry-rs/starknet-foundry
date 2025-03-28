use starknet::ContractAddress;

#[starknet::interface]
trait IBlocker<TContractState> {
    fn write_block(ref self: TContractState);
    fn read_block_number(self: @TContractState) -> u64;
    fn read_block_timestamp(self: @TContractState) -> u64;
    fn read_sequencer_address(self: @TContractState) -> ContractAddress;
    fn read_block_hash(self: @TContractState) -> felt252;
}

#[starknet::contract]
mod Blocker {
    use core::{array::ArrayTrait, box::BoxTrait};
    use starknet::{
        SyscallResultTrait, syscalls::get_block_hash_syscall, get_block_info, ContractAddress,
    };
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        block_number: u64,
        block_timestamp: u64,
        block_hash: felt252,
        sequencer_address: ContractAddress,
    }

    #[abi(embed_v0)]
    impl IBlockerImpl of super::IBlocker<ContractState> {
        fn write_block(ref self: ContractState) {
            let block_info = get_block_info().unbox();
            self.block_number.write(block_info.block_number);
            self.block_timestamp.write(block_info.block_timestamp);
            self.sequencer_address.write(block_info.sequencer_address);

            let block_hash = get_block_hash_syscall(block_info.block_number - 10).unwrap_syscall();
            self.block_hash.write(block_hash);
        }

        fn read_block_number(self: @ContractState) -> u64 {
            self.block_number.read()
        }
        fn read_block_timestamp(self: @ContractState) -> u64 {
            self.block_timestamp.read()
        }
        fn read_sequencer_address(self: @ContractState) -> ContractAddress {
            self.sequencer_address.read()
        }
        fn read_block_hash(self: @ContractState) -> felt252 {
            self.block_hash.read()
        }
    }
}
