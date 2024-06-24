use starknet::ContractAddress;

#[starknet::interface]
trait IRollChecker<TContractState> {
    fn get_block_number(self: @TContractState) -> u64;
}

#[starknet::interface]
trait IRollCheckerProxy<TContractState> {
    fn get_roll_checkers_block_number(self: @TContractState, address: ContractAddress) -> u64;
    fn get_block_number(self: @TContractState) -> u64;
    fn call_proxy(self: @TContractState, address: ContractAddress) -> (u64, u64);
}

#[starknet::contract]
mod RollCheckerProxy {
    use starknet::ContractAddress;
    use super::IRollCheckerDispatcherTrait;
    use super::IRollCheckerDispatcher;
    use super::IRollCheckerProxyDispatcherTrait;
    use super::IRollCheckerProxyDispatcher;
    use starknet::{get_contract_address};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IRollCheckerProxy of super::IRollCheckerProxy<ContractState> {
        fn get_roll_checkers_block_number(self: @ContractState, address: ContractAddress) -> u64 {
            let roll_checker = IRollCheckerDispatcher { contract_address: address };
            roll_checker.get_block_number()
        }

        fn get_block_number(self: @ContractState) -> u64 {
            starknet::get_block_info().unbox().block_number
        }

        fn call_proxy(self: @ContractState, address: ContractAddress) -> (u64, u64) {
            let dispatcher = IRollCheckerProxyDispatcher { contract_address: address };
            let block_number = self.get_block_number();
            let res = dispatcher.get_roll_checkers_block_number(get_contract_address());
            (block_number, res)
        }
    }
}
