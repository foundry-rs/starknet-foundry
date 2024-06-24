use starknet::ContractAddress;

#[starknet::interface]
trait IWarpChecker<TContractState> {
    fn get_block_timestamp(self: @TContractState) -> u64;
}

#[starknet::interface]
trait IWarpCheckerProxy<TContractState> {
    fn get_warp_checkers_block_timestamp(self: @TContractState, address: ContractAddress) -> u64;
    fn get_block_timestamp(self: @TContractState) -> u64;
    fn call_proxy(self: @TContractState, address: ContractAddress) -> (u64, u64);
}

#[starknet::contract]
mod WarpCheckerProxy {
    use starknet::ContractAddress;
    use super::IWarpCheckerDispatcherTrait;
    use super::IWarpCheckerDispatcher;
    use super::IWarpCheckerProxyDispatcher;
    use super::IWarpCheckerProxyDispatcherTrait;
    use starknet::get_contract_address;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IWarpCheckerProxy of super::IWarpCheckerProxy<ContractState> {
        fn get_warp_checkers_block_timestamp(
            self: @ContractState, address: ContractAddress
        ) -> u64 {
            let warp_checker = IWarpCheckerDispatcher { contract_address: address };
            warp_checker.get_block_timestamp()
        }

        fn get_block_timestamp(self: @ContractState) -> u64 {
            starknet::get_block_info().unbox().block_timestamp
        }

        fn call_proxy(self: @ContractState, address: ContractAddress) -> (u64, u64) {
            let dispatcher = IWarpCheckerProxyDispatcher { contract_address: address };
            let timestamp = self.get_block_timestamp();
            let res = dispatcher.get_warp_checkers_block_timestamp(get_contract_address());
            (timestamp, res)
        }
    }
}
