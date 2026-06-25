#[starknet::interface]
pub trait IPanicking<TContractState> {
    fn panic_with_byte_array(self: @TContractState);
}

#[starknet::contract]
pub mod PanickingContract {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl PanickingImpl of super::IPanicking<ContractState> {
        fn panic_with_byte_array(self: @ContractState) {
            panic!("This will panic");
        }
    }
}
