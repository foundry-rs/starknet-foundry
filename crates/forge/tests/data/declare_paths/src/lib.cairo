// Contract named `HelloStarknet` defined in `src/`. A second contract with the same name lives
// in `tests/`, making `declare("HelloStarknet")` ambiguous.
#[starknet::contract]
pub mod HelloStarknet {
    #[storage]
    struct Storage {
        balance: felt252,
    }
}
