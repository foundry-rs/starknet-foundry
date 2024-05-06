use starknet::ContractAddress;

#[starknet::interface]
trait ICheatBlockNumberChecker<TContractState> {
    fn get_block_number(self: @TContractState) -> u64;
}

#[starknet::interface]
trait ICheatBlockNumberCheckerProxy<TContractState> {
    fn get_cheated_block_number(self: @TContractState, address: ContractAddress) -> u64;
    fn get_block_number(self: @TContractState) -> u64;
    fn call_proxy(self: @TContractState, address: ContractAddress) -> (u64, u64);
}

#[starknet::contract]
mod CheatBlockNumberCheckerProxy {
    use starknet::ContractAddress;
    use super::ICheatBlockNumberCheckerDispatcherTrait;
    use super::ICheatBlockNumberCheckerDispatcher;
    use super::ICheatBlockNumberCheckerProxyDispatcherTrait;
    use super::ICheatBlockNumberCheckerProxyDispatcher;
    use starknet::{get_contract_address};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatBlockNumberCheckerProxy of super::ICheatBlockNumberCheckerProxy<ContractState> {
        fn get_cheated_block_number(self: @ContractState, address: ContractAddress) -> u64 {
            let cheat_block_number_checker = ICheatBlockNumberCheckerDispatcher {
                contract_address: address
            };
            cheat_block_number_checker.get_block_number()
        }

        fn get_block_number(self: @ContractState) -> u64 {
            starknet::get_block_info().unbox().block_number
        }

        fn call_proxy(self: @ContractState, address: ContractAddress) -> (u64, u64) {
            let dispatcher = ICheatBlockNumberCheckerProxyDispatcher { contract_address: address };
            let block_number = self.get_block_number();
            let res = dispatcher.get_cheated_block_number(get_contract_address());
            (block_number, res)
        }
    }
}
