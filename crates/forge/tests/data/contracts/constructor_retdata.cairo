#[starknet::contract]
mod ConstructorRetdata {
    use array::ArrayTrait;

    #[storage]
    struct Storage {}

    #[constructor]
    fn constructor(ref self: ContractState) -> Span<felt252> {
        array![2, 3, 4].span()
    }
}
