use starknet::ContractAddress;

#[starknet::interface]
trait IBlockInfoChecker<TContractState> {
    fn read_block_number(self: @TContractState) -> u64;
    fn read_block_timestamp(self: @TContractState) -> u64;
    fn read_sequencer_address(self: @TContractState) -> ContractAddress;
}

#[starknet::contract]
mod BlockInfoChecker {
    use core::starknet::SyscallResultTrait;
    use array::ArrayTrait;
    use starknet::get_block_info;
    use box::BoxTrait;
    use starknet::ContractAddress;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IBlockInfoChecker of super::IBlockInfoChecker<ContractState> {
        fn read_block_number(self: @ContractState) -> u64 {
            get_block_info().unbox().block_number
        }
        fn read_block_timestamp(self: @ContractState) -> u64 {
            get_block_info().unbox().block_timestamp
        }
        fn read_sequencer_address(self: @ContractState) -> ContractAddress {
            get_block_info().unbox().sequencer_address
        }
    }
}
