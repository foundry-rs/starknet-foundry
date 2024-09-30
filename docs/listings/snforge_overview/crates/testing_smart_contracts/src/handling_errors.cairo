#[starknet::interface]
pub trait IPanicContract<TContractState> {
    fn do_a_panic(self: @TContractState);
    fn do_a_string_panic(self: @TContractState);
}

#[starknet::contract]
pub mod PanicContract {
    use core::array::ArrayTrait;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    pub impl PanicContractImpl of super::IPanicContract<ContractState> {
        // Panics
        fn do_a_panic(self: @ContractState) {
            panic(array!['PANIC', 'DAYTAH']);
        }

        fn do_a_string_panic(self: @ContractState) {
            // A macro which allows panicking with a ByteArray (string) instance
            panic!("This is panicking with a string, which can be longer than 31 characters");
        }
    }
}
