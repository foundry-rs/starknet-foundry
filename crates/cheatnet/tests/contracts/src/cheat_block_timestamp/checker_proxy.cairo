use starknet::ContractAddress;

#[starknet::interface]
trait ICheatBlockTimestampChecker<TContractState> {
    fn get_block_timestamp(self: @TContractState) -> u64;
}

#[starknet::interface]
trait ICheatBlockTimestampCheckerProxy<TContractState> {
    fn get_cheated_block_timestamp(self: @TContractState, address: ContractAddress) -> u64;
    fn get_block_timestamp(self: @TContractState) -> u64;
    fn call_proxy(self: @TContractState, address: ContractAddress) -> (u64, u64);
}

#[starknet::contract]
mod CheatBlockTimestampCheckerProxy {
    use starknet::{ContractAddress, get_contract_address};
    use super::{
        ICheatBlockTimestampCheckerDispatcher, ICheatBlockTimestampCheckerDispatcherTrait,
        ICheatBlockTimestampCheckerProxyDispatcher, ICheatBlockTimestampCheckerProxyDispatcherTrait,
    };

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatBlockTimestampCheckerProxy of super::ICheatBlockTimestampCheckerProxy<
        ContractState,
    > {
        fn get_cheated_block_timestamp(self: @ContractState, address: ContractAddress) -> u64 {
            let cheat_block_timestamp_checker = ICheatBlockTimestampCheckerDispatcher {
                contract_address: address,
            };
            cheat_block_timestamp_checker.get_block_timestamp()
        }

        fn get_block_timestamp(self: @ContractState) -> u64 {
            starknet::get_block_info().unbox().block_timestamp
        }

        fn call_proxy(self: @ContractState, address: ContractAddress) -> (u64, u64) {
            let dispatcher = ICheatBlockTimestampCheckerProxyDispatcher {
                contract_address: address,
            };
            let timestamp = self.get_block_timestamp();
            let res = dispatcher.get_cheated_block_timestamp(get_contract_address());
            (timestamp, res)
        }
    }
}
