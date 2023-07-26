#[starknet::interface]
trait IWarpChecker<TContractState> {
    fn get_block_timestamp(ref self: TContractState) -> u64;
}

#[starknet::contract]
mod WarpChecker {
    use box::BoxTrait;

    #[storage]
    struct Storage {}

    #[external(v0)]
    impl WarpChecker of super::IWarpChecker<ContractState> {
        fn get_block_timestamp(ref self: ContractState) -> u64 {
            starknet::get_block_info().unbox().block_timestamp
        }
    }
}
