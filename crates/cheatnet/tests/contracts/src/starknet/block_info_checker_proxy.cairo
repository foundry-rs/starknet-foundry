use starknet::ContractAddress;

#[starknet::interface]
trait IBlockInfoChecker<TContractState> {
    fn read_block_number(self: @TContractState) -> u64;
    fn read_block_timestamp(self: @TContractState) -> u64;
    fn read_sequencer_address(self: @TContractState) -> ContractAddress;
}

#[starknet::interface]
trait IBlockInfoCheckerProxy<TContractState> {
    fn read_block_number(ref self: TContractState, address: ContractAddress) -> u64;
    fn read_block_timestamp(ref self: TContractState, address: ContractAddress) -> u64;
    fn read_sequencer_address(
        ref self: TContractState, address: ContractAddress,
    ) -> ContractAddress;
}

#[starknet::contract]
mod BlockInfoCheckerProxy {
    use starknet::ContractAddress;
    use super::IBlockInfoCheckerDispatcherTrait;
    use super::IBlockInfoCheckerDispatcher;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IBlockInfoCheckerProxy of super::IBlockInfoCheckerProxy<ContractState> {
        fn read_block_number(ref self: ContractState, address: ContractAddress) -> u64 {
            let block_info_checker = IBlockInfoCheckerDispatcher { contract_address: address };
            block_info_checker.read_block_number()
        }
        fn read_block_timestamp(ref self: ContractState, address: ContractAddress) -> u64 {
            let block_info_checker = IBlockInfoCheckerDispatcher { contract_address: address };
            block_info_checker.read_block_timestamp()
        }
        fn read_sequencer_address(
            ref self: ContractState, address: ContractAddress,
        ) -> ContractAddress {
            let block_info_checker = IBlockInfoCheckerDispatcher { contract_address: address };
            block_info_checker.read_sequencer_address()
        }
    }
}
