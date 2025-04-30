use starknet::ContractAddress;

#[starknet::interface]
trait ICheatBlockHashChecker<TContractState> {
    fn get_block_hash(ref self: TContractState, block_number: u64) -> felt252;
}

#[starknet::interface]
trait ICheatBlockHashCheckerProxy<TContractState> {
    fn get_cheated_block_hash(
        self: @TContractState, address: ContractAddress, block_number: u64,
    ) -> felt252;
    fn get_block_hash(self: @TContractState, block_number: u64) -> felt252;
    fn call_proxy(
        self: @TContractState, address: ContractAddress, block_number: u64,
    ) -> (felt252, felt252);
}

#[starknet::contract]
mod CheatBlockHashCheckerProxy {
    use starknet::ContractAddress;
    use super::ICheatBlockHashCheckerDispatcherTrait;
    use super::ICheatBlockHashCheckerDispatcher;
    use super::ICheatBlockHashCheckerProxyDispatcher;
    use super::ICheatBlockHashCheckerProxyDispatcherTrait;
    use starknet::get_contract_address;
    use starknet::syscalls::get_block_hash_syscall;
    use starknet::SyscallResultTrait;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatBlockHashCheckerProxy of super::ICheatBlockHashCheckerProxy<ContractState> {
        fn get_cheated_block_hash(
            self: @ContractState, address: ContractAddress, block_number: u64,
        ) -> felt252 {
            let cheat_block_hash_checker = ICheatBlockHashCheckerDispatcher {
                contract_address: address,
            };
            cheat_block_hash_checker.get_block_hash(block_number)
        }

        fn get_block_hash(self: @ContractState, block_number: u64) -> felt252 {
            let block_hash = get_block_hash_syscall(block_number).unwrap_syscall();

            block_hash
        }

        fn call_proxy(
            self: @ContractState, address: ContractAddress, block_number: u64,
        ) -> (felt252, felt252) {
            let dispatcher = ICheatBlockHashCheckerProxyDispatcher { contract_address: address };
            let hash = self.get_block_hash(block_number);
            let res = dispatcher.get_cheated_block_hash(get_contract_address(), block_number);
            (hash, res)
        }
    }
}
