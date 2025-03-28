#[starknet::interface]
trait ICheatBlockTimestampChecker<TContractState> {
    fn get_block_timestamp(ref self: TContractState) -> u64;
    fn get_block_timestamp_and_emit_event(ref self: TContractState) -> u64;
    fn get_block_timestamp_and_number(ref self: TContractState) -> (u64, u64);
}

#[starknet::contract]
mod CheatBlockTimestampChecker {
    use box::BoxTrait;

    #[storage]
    struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        BlockTimestampEmitted: BlockTimestampEmitted,
    }

    #[derive(Drop, starknet::Event)]
    struct BlockTimestampEmitted {
        block_timestamp: u64,
    }


    #[abi(embed_v0)]
    impl CheatBlockTimestampChecker of super::ICheatBlockTimestampChecker<ContractState> {
        fn get_block_timestamp(ref self: ContractState) -> u64 {
            starknet::get_block_info().unbox().block_timestamp
        }

        fn get_block_timestamp_and_emit_event(ref self: ContractState) -> u64 {
            let block_timestamp = starknet::get_block_info().unbox().block_timestamp;
            self.emit(Event::BlockTimestampEmitted(BlockTimestampEmitted { block_timestamp }));
            block_timestamp
        }

        fn get_block_timestamp_and_number(ref self: ContractState) -> (u64, u64) {
            let block_info = starknet::get_block_info().unbox();

            (block_info.block_timestamp, block_info.block_number)
        }
    }
}
