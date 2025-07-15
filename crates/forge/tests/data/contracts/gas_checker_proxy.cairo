use starknet::{ContractAddress, SyscallResult};

#[starknet::interface]
trait IGasChecker<TContractState> {
    fn send_l1_message(self: @TContractState);
}


#[starknet::interface]
trait IGasCheckerProxy<TContractState> {
    fn send_l1_message_from_gas_checker(self: @TContractState, address: ContractAddress);
    fn call_other_contract(
        self: @TContractState,
        contract_address: ContractAddress,
        entry_point_selector: felt252,
        calldata: Array::<felt252>,
    ) -> SyscallResult<Span<felt252>>;
}

#[starknet::contract]
mod GasCheckerProxy {
    use starknet::{ContractAddress, SyscallResult};
    use starknet::syscalls::call_contract_syscall;
    use super::{IGasCheckerDispatcher, IGasCheckerDispatcherTrait};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IGasCheckerProxy of super::IGasCheckerProxy<ContractState> {
        fn send_l1_message_from_gas_checker(self: @ContractState, address: ContractAddress) {
            let gas_checker = IGasCheckerDispatcher { contract_address: address };
            gas_checker.send_l1_message()
        }

        fn call_other_contract(
            self: @ContractState,
            contract_address: ContractAddress,
            entry_point_selector: felt252,
            calldata: Array::<felt252>,
        ) -> SyscallResult<Span<felt252>> {
            call_contract_syscall(contract_address, entry_point_selector, calldata.span())
        }
    }
}
