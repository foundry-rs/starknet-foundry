use starknet::{ContractAddress, ClassHash};

#[starknet::interface]
trait ITraceInfoProxy<T> {
    fn with_libcall(self: @T, class_hash: ClassHash) -> felt252;
    fn regular_call(self: @T, contract_address: ContractAddress) -> felt252;
    fn with_panic(self: @T, contract_address: ContractAddress);
    fn call_two(self: @T, checker_address: ContractAddress, dummy_address: ContractAddress);
}

#[starknet::interface]
trait ITraceInfoChecker<T> {
    fn from_proxy(self: @T, data: felt252) -> felt252;
    fn panic(self: @T);
}

#[starknet::contract]
mod TraceInfoChecker {
    use super::{ITraceInfoChecker, ITraceInfoProxyDispatcher, ITraceInfoProxyDispatcherTrait};
    use starknet::{ContractAddress, get_contract_address};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ITraceInfoChceckerImpl of ITraceInfoChecker<ContractState> {
        fn from_proxy(self: @ContractState, data: felt252) -> felt252 {
            100 + data
        }

        fn panic(self: @ContractState) {
            panic_with_felt252('panic');
        }
    }

    #[l1_handler]
    fn handle_l1_message(
        ref self: ContractState, from_address: felt252, proxy_address: ContractAddress
    ) -> felt252 {
        ITraceInfoProxyDispatcher { contract_address: proxy_address }
            .regular_call(get_contract_address())
    }
}
