use starknet::ContractAddress;

#[starknet::interface]
trait IGasChecker<TContractState> {
    fn send_l1_message(self: @TContractState);
}


#[starknet::interface]
trait IGasCheckerProxy<TContractState> {
    fn send_l1_message_from_gas_checker(self: @TContractState, address: ContractAddress);
}

#[starknet::contract]
mod GasCheckerProxy {
    use starknet::ContractAddress;
    use super::IGasCheckerDispatcherTrait;
    use super::IGasCheckerDispatcher;

    #[storage]
    struct Storage {}

    #[external(v0)]
    impl IGasCheckerProxy of super::IGasCheckerProxy<ContractState> {
        fn send_l1_message_from_gas_checker(
            self: @ContractState,
            address: ContractAddress)
        {
            let gas_checker = IGasCheckerDispatcher { contract_address: address };
            gas_checker.send_l1_message()
        }
    }
}
