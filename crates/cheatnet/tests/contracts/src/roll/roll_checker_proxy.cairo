use starknet::ContractAddress;

#[starknet::interface]
trait IRollChecker<TContractState> {
    fn get_block_number(ref self: TContractState) -> u64;
}

#[starknet::interface]
trait IRollCheckerProxy<TContractState> {
    fn get_roll_checkers_block_number(ref self: TContractState, address: ContractAddress) -> u64;
}

#[starknet::contract]
mod RollCheckerProxy {
    use starknet::ContractAddress;
    use super::IRollCheckerDispatcherTrait;
    use super::IRollCheckerDispatcher;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IRollCheckerProxy of super::IRollCheckerProxy<ContractState> {
        fn get_roll_checkers_block_number(
            ref self: ContractState, address: ContractAddress
        ) -> u64 {
            let roll_checker = IRollCheckerDispatcher { contract_address: address };
            roll_checker.get_block_number()
        }
    }
}
