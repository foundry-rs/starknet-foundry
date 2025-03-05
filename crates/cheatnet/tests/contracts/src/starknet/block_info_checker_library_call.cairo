use starknet::ClassHash;
use starknet::ContractAddress;

#[starknet::interface]
trait IBlockInfoChecker<TContractState> {
    fn read_block_number(ref self: TContractState) -> u64;
    fn read_block_timestamp(ref self: TContractState) -> u64;
    fn read_sequencer_address(ref self: TContractState) -> ContractAddress;
}

#[starknet::interface]
trait IBlockInfoCheckerLibCall<TContractState> {
    fn read_block_number_with_lib_call(ref self: TContractState, class_hash: ClassHash) -> u64;
    fn read_block_timestamp_with_lib_call(ref self: TContractState, class_hash: ClassHash) -> u64;
    fn read_sequencer_address_with_lib_call(
        ref self: TContractState, class_hash: ClassHash,
    ) -> ContractAddress;
}

#[starknet::contract]
mod BlockInfoCheckerLibCall {
    use super::{IBlockInfoCheckerDispatcherTrait, IBlockInfoCheckerLibraryDispatcher};
    use starknet::ClassHash;
    use starknet::ContractAddress;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IBlockInfoCheckerLibCall of super::IBlockInfoCheckerLibCall<ContractState> {
        fn read_block_number_with_lib_call(ref self: ContractState, class_hash: ClassHash) -> u64 {
            let block_info_checker = IBlockInfoCheckerLibraryDispatcher { class_hash };
            block_info_checker.read_block_number()
        }
        fn read_block_timestamp_with_lib_call(
            ref self: ContractState, class_hash: ClassHash,
        ) -> u64 {
            let block_info_checker = IBlockInfoCheckerLibraryDispatcher { class_hash };
            block_info_checker.read_block_timestamp()
        }
        fn read_sequencer_address_with_lib_call(
            ref self: ContractState, class_hash: ClassHash,
        ) -> ContractAddress {
            let block_info_checker = IBlockInfoCheckerLibraryDispatcher { class_hash };
            block_info_checker.read_sequencer_address()
        }
    }
}
