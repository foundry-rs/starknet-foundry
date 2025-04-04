pub fn subtract(a: felt252, b: felt252) -> felt252 {
    a - b
}

#[starknet::interface]
trait ISubtractionContract<TContractState> {
    fn answer(ref self: TContractState) -> felt252;
}

#[starknet::contract]
mod SubtractionContract {
    use super::subtract;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl SubtractionContractImpl of super::ISubtractionContract<ContractState> {
        fn answer(ref self: ContractState) -> felt252 {
            subtract(10, 20)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::subtract;

    #[test]
    fn it_works() {
        assert(subtract(3, 2) == 1, 'it works!');
    }
}
