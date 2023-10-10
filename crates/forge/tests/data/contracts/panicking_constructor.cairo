#[starknet::contract]
mod PanickingConstructor {
    use array::ArrayTrait;

    #[storage]
    struct Storage {}

    #[constructor]
    fn constructor(ref self: ContractState) {
        let mut panic_data = ArrayTrait::new();
        panic_data.append('PANIK');
        panic_data.append('DEJTA');
        panic(panic_data);
    }
}
