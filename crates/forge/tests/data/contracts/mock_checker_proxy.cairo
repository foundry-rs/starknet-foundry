use starknet::ContractAddress;

#[starknet::interface]
trait IMockChecker<TContractState> {
    fn get_thing(self: @TContractState) -> felt252;
}


#[starknet::interface]
trait IMockCheckerProxy<TContractState> {
    fn get_thing_from_contract(ref self: TContractState, address: ContractAddress) -> felt252;
}

#[starknet::contract]
mod MockCheckerProxy {
    use starknet::ContractAddress;
    use super::IMockCheckerDispatcherTrait;
    use super::IMockCheckerDispatcher;

    #[storage]
    struct Storage {}

    #[external(v0)]
    impl IMockCheckerProxy of super::IMockCheckerProxy<ContractState> {
        fn get_thing_from_contract(ref self: ContractState, address: ContractAddress) -> felt252 {
            let dispatcher = IMockCheckerDispatcher { contract_address: address };
            dispatcher.get_thing()
        }
    }
}
