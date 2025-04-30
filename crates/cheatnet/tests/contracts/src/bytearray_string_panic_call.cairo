use starknet::ContractAddress;

#[starknet::interface]
trait IByteArrayPanickingContract<TContractState> {
    fn do_panic(self: @TContractState);
    fn do_panic_felts(self: @TContractState, data: Array<felt252>);
}

#[starknet::contract]
mod ByteArrayPanickingContract {
    use core::panics::panic_with_byte_array;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl Impl of super::IByteArrayPanickingContract<ContractState> {
        fn do_panic(self: @ContractState) {
            let data =
                "This is a very long\n and multi line string, that will for sure saturate the pending_word";

            panic_with_byte_array(@data);
        }

        fn do_panic_felts(self: @ContractState, data: Array<felt252>) {
            panic(data);
        }
    }
}

#[starknet::interface]
trait IByteArrayPanickingContractProxy<TContractState> {
    fn call_bytearray_panicking_contract(self: @TContractState, contract_address: ContractAddress);
    fn call_felts_panicking_contract(
        self: @TContractState, contract_address: ContractAddress, data: Array<felt252>,
    );
}

#[starknet::contract]
mod ByteArrayPanickingContractProxy {
    use starknet::ContractAddress;
    use super::{IByteArrayPanickingContractDispatcherTrait, IByteArrayPanickingContractDispatcher};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl Impl of super::IByteArrayPanickingContractProxy<ContractState> {
        fn call_bytearray_panicking_contract(
            self: @ContractState, contract_address: ContractAddress,
        ) {
            IByteArrayPanickingContractDispatcher { contract_address }.do_panic();
        }

        fn call_felts_panicking_contract(
            self: @ContractState, contract_address: ContractAddress, data: Array<felt252>,
        ) {
            IByteArrayPanickingContractDispatcher { contract_address }.do_panic_felts(data);
        }
    }
}
