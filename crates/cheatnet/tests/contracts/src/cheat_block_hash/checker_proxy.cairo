use starknet::ContractAddress;

#[starknet::interface]
trait ICheatBlockHashChecker<TContractState> {
    fn get_block_hash(self: @TContractState) -> felt252;
}

#[starknet::interface]
trait ICheatBlockHashCheckerProxy<TContractState> {
    fn get_cheated_block_hash(self: @TContractState, address: ContractAddress) -> felt252;
    fn get_block_hash(self: @TContractState) -> felt252;
    fn call_proxy(self: @TContractState, address: ContractAddress) -> (felt252, felt252);
}

#[starknet::contract]
mod CheatBlockHashCheckerProxy {
    use starknet::ContractAddress;
    use super::ICheatBlockHashCheckerDispatcherTrait;
    use super::ICheatBlockHashCheckerDispatcher;
    use super::ICheatBlockHashCheckerProxyDispatcher;
    use super::ICheatBlockHashCheckerProxyDispatcherTrait;
    use starknet::get_contract_address;
    use starknet::{get_block_info, get_block_hash_syscall};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatBlockHashCheckerProxy of super::ICheatBlockHashCheckerProxy<ContractState> {
        fn get_cheated_block_hash(self: @ContractState, address: ContractAddress) -> felt252 {
            let cheat_block_hash_checker = ICheatBlockHashCheckerDispatcher {
                contract_address: address
            };
            cheat_block_hash_checker.get_block_hash()
        }

        fn get_block_hash(self: @ContractState) -> felt252 {
            let block_info = get_block_info().unbox();
            let block_hash = get_block_hash_syscall(block_info.block_number).unwrap_syscall();

            block_hash
        }

        fn call_proxy(self: @ContractState, address: ContractAddress) -> (felt252, felt252) {
            let dispatcher = ICheatBlockHashCheckerProxyDispatcher { contract_address: address };
            let hash = self.get_block_hash();
            let res = dispatcher.get_cheated_block_hash(get_contract_address());
            (hash, res)
        }
    }
}
