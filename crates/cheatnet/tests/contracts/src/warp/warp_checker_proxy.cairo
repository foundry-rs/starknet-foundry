use starknet::ContractAddress;

#[starknet::interface]
trait IWarpChecker<TContractState> {
    fn get_block_timestamp(ref self: TContractState) -> u64;
}

#[starknet::interface]
trait IWarpCheckerProxy<TContractState> {
    fn get_warp_checkers_block_timestamp(ref self: TContractState, address: ContractAddress) -> u64;
}

#[starknet::contract]
mod WarpCheckerProxy {
    use starknet::ContractAddress;
    use super::IWarpCheckerDispatcherTrait;
    use super::IWarpCheckerDispatcher;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IWarpCheckerProxy of super::IWarpCheckerProxy<ContractState> {
        fn get_warp_checkers_block_timestamp(
            ref self: ContractState, address: ContractAddress
        ) -> u64 {
            let warp_checker = IWarpCheckerDispatcher { contract_address: address };
            warp_checker.get_block_timestamp()
        }
    }
}
