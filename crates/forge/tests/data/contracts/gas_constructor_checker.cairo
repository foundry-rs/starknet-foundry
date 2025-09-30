#[starknet::contract]
mod GasConstructorChecker {
    #[storage]
    struct Storage {}

    #[constructor]
    fn constructor(ref self: ContractState, compute_keccak: bool) {
        if compute_keccak {
            keccak::keccak_u256s_le_inputs(array![1].span());
            keccak::keccak_u256s_le_inputs(array![1].span());
        }
    }
}
