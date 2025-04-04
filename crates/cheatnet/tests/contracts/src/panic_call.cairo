#[starknet::contract]
mod PanicCall {
    #[storage]
    struct Storage {}

    #[external(v0)]
    fn panic_call(ref self: ContractState) {
        panic(
            array![
                'shortstring',
                0,
                0x800000000000011000000000000000000000000000000000000000000000000,
                'shortstring2',
            ],
        );
    }
}
