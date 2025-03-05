use starknet::{ContractAddress, ClassHash};

#[starknet::interface]
pub trait ITraceInfoProxy<T> {
    fn with_libcall(
        ref self: T, class_hash: ClassHash, empty_hash: ClassHash, salt: felt252,
    ) -> felt252;
    fn regular_call(
        ref self: T, contract_address: ContractAddress, empty_hash: ClassHash, salt: felt252,
    ) -> felt252;
    fn with_panic(
        ref self: T, contract_address: ContractAddress, empty_hash: ClassHash, salt: felt252,
    );
    fn call_two(
        ref self: T,
        checker_address: ContractAddress,
        dummy_address: ContractAddress,
        empty_hash: ClassHash,
        salt: felt252,
    );
}

#[starknet::contract]
mod TraceInfoProxy {
    use super::ITraceInfoProxy;
    use trace_resources::trace_info_checker::{
        ITraceInfoCheckerDispatcherTrait, ITraceInfoCheckerDispatcher,
        ITraceInfoCheckerLibraryDispatcher,
    };
    use trace_resources::trace_dummy::{ITraceDummyDispatcher, ITraceDummyDispatcherTrait};
    use starknet::{ContractAddress, ClassHash};
    use super::super::use_builtins_and_syscalls;

    #[storage]
    struct Storage {
        balance: u8,
    }

    #[constructor]
    fn constructor(
        ref self: ContractState,
        contract_address: ContractAddress,
        empty_hash: ClassHash,
        salt: felt252,
    ) {
        use_builtins_and_syscalls(empty_hash, salt);

        ITraceInfoCheckerDispatcher { contract_address }.from_proxy(1, empty_hash, 10 * salt);
    }

    #[abi(embed_v0)]
    impl ITraceInfoProxyImpl of ITraceInfoProxy<ContractState> {
        fn regular_call(
            ref self: ContractState,
            contract_address: ContractAddress,
            empty_hash: ClassHash,
            salt: felt252,
        ) -> felt252 {
            use_builtins_and_syscalls(empty_hash, salt);

            ITraceInfoCheckerDispatcher { contract_address }.from_proxy(2, empty_hash, 10 * salt)
        }

        fn with_libcall(
            ref self: ContractState, class_hash: ClassHash, empty_hash: ClassHash, salt: felt252,
        ) -> felt252 {
            use_builtins_and_syscalls(empty_hash, salt);

            ITraceInfoCheckerLibraryDispatcher { class_hash }.from_proxy(3, empty_hash, 10 * salt)
        }

        fn with_panic(
            ref self: ContractState,
            contract_address: ContractAddress,
            empty_hash: ClassHash,
            salt: felt252,
        ) {
            use_builtins_and_syscalls(empty_hash, salt);

            ITraceInfoCheckerDispatcher { contract_address }.panic(empty_hash, 10 * salt);
            // unreachable code to check if we stop executing after panic
            ITraceInfoCheckerDispatcher { contract_address }.from_proxy(5, empty_hash, 20 * salt);
        }

        fn call_two(
            ref self: ContractState,
            checker_address: ContractAddress,
            dummy_address: ContractAddress,
            empty_hash: ClassHash,
            salt: felt252,
        ) {
            ITraceInfoCheckerDispatcher { contract_address: checker_address }
                .from_proxy(42, empty_hash, 10 * salt);

            use_builtins_and_syscalls(empty_hash, salt);

            ITraceDummyDispatcher { contract_address: dummy_address }
                .from_proxy_dummy(empty_hash, 20 * salt);
        }
    }
}
