#[starknet::interface]
trait IConstructorWarpChecker<TContractState> {
    fn get_stored_block_timestamp(ref self: TContractState) -> u64;
}

#[starknet::contract]
mod ConstructorWarpChecker {
    use box::BoxTrait;
    #[storage]
    struct Storage {
        blk_timestamp: u64,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        let blk_timestamp = starknet::get_block_info().unbox().block_timestamp;
        self.blk_timestamp.write(blk_timestamp);
    }

    #[abi(embed_v0)]
    impl IConstructorWarpChecker of super::IConstructorWarpChecker<ContractState> {
        fn get_stored_block_timestamp(ref self: ContractState) -> u64 {
            self.blk_timestamp.read()
        }
    }
}
