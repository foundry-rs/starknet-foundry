#[starknet::interface]
trait ITraceInfoProxy<T> {
    fn with_libcall(ref self: T, class_hash: starknet::ClassHash) -> felt252;
    fn regular_call(self: @T, contract_address: starknet::ContractAddress) -> felt252;
}

#[starknet::interface]
trait ITraceInfoChecker<T> {
    fn from_proxy(self: @T, data: felt252) -> felt252;
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
    }

    #[l1_handler]
    fn handle_l1_message(
        ref self: ContractState, from_address: felt252, proxy_address: ContractAddress
    ) -> felt252 {
        ITraceInfoProxyDispatcher { contract_address: proxy_address }
            .regular_call(get_contract_address())
    }
}
