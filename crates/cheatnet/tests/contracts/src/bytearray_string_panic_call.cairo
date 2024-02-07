use starknet::ContractAddress;

#[starknet::interface]
trait IByteArrayPanickingContract<TContractState> {
    fn do_panic(self: @TContractState);
}

// ByteArrayPanickingContract
#[starknet::contract]
mod BAPC {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl Impl of super::IByteArrayPanickingContract<ContractState> {
        fn do_panic(self: @ContractState) {
            assert!(
                false,
                "This is a very long\n and multiline string, that will for sure saturate the pending_word"
            );
        }
    }
}

#[starknet::interface]
trait IProxyForBAPC<TContractState> {
    fn call_bytearray_panicking_contract(self: @TContractState, contract_address: ContractAddress);
}

#[starknet::contract]
mod ProxyForBAPC {
    use starknet::ContractAddress;
    use super::{IByteArrayPanickingContractDispatcherTrait, IByteArrayPanickingContractDispatcher};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl Impl of super::IProxyForBAPC<ContractState> {
        fn call_bytearray_panicking_contract(
            self: @ContractState, contract_address: ContractAddress
        ) {
            IByteArrayPanickingContractDispatcher { contract_address }.do_panic();
        }
    }
}
