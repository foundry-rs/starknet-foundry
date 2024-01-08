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

#[starknet::interface]
trait ITraceDummy<T> {
    fn from_proxy_dummy(ref self: T);
}

#[starknet::contract]
mod TraceInfoProxy {
    use super::{
        ITraceInfoCheckerDispatcherTrait, ITraceInfoCheckerDispatcher,
        ITraceInfoCheckerLibraryDispatcher, ITraceInfoProxy,
        ITraceDummyDispatcher, ITraceDummyDispatcherTrait
    };
    use starknet::{ContractAddress, ClassHash};

    #[storage]
    struct Storage {}

    #[constructor]
    fn constructor(ref self: ContractState, contract_address: ContractAddress) {
        ITraceInfoCheckerDispatcher { contract_address }.from_proxy(1);
    }

    #[abi(embed_v0)]
    impl ITraceInfoProxyImpl of ITraceInfoProxy<ContractState> {
        fn regular_call(self: @ContractState, contract_address: ContractAddress) -> felt252 {
            ITraceInfoCheckerDispatcher { contract_address }.from_proxy(2)
        }

        fn with_libcall(self: @ContractState, class_hash: ClassHash) -> felt252 {
            ITraceInfoCheckerLibraryDispatcher { class_hash }.from_proxy(3)
        }

        fn with_panic(self: @ContractState, contract_address: ContractAddress) {
            ITraceInfoCheckerDispatcher { contract_address }.panic();
            // unreachable code to check if we stop executing after panic
            ITraceInfoCheckerDispatcher { contract_address }.from_proxy(5);
        }

        fn call_two(
            self: @ContractState, checker_address: ContractAddress, dummy_address: ContractAddress
        ) {
            ITraceInfoCheckerDispatcher { contract_address: checker_address }.from_proxy(42);
            ITraceDummyDispatcher { contract_address: dummy_address }.from_proxy_dummy();
        }
    }
}
