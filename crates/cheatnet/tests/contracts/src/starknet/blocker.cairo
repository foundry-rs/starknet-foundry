use starknet::ContractAddress;

#[starknet::interface]
trait IBlocker<TContractState> {
    fn write_block(ref self: TContractState);
    fn read_block_number(self: @TContractState) -> u64;
    fn read_block_timestamp(self: @TContractState) -> u64;
    fn read_sequencer_address(self: @TContractState) -> ContractAddress;
}

#[starknet::contract]
mod Blocker {
    use array::ArrayTrait;
    use starknet::get_block_info;
    use box::BoxTrait;
    use starknet::ContractAddress;

    #[storage]
    struct Storage {
        block_number: u64,
        block_timestamp: u64,
        sequencer_address: ContractAddress,
    }

    #[external(v0)]
    impl IBlockerImpl of super::IBlocker<ContractState> {
        fn write_block(ref self: ContractState) {
            let block_info = get_block_info().unbox();
            self.block_number.write(block_info.block_number);
            self.block_timestamp.write(block_info.block_timestamp);
            self.sequencer_address.write(block_info.sequencer_address);
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
    }
}
