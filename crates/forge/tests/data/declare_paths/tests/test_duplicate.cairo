use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::declare;

// Second contract sharing the name `HelloStarknet` with the one in `src/`, but with a distinct
// fully qualified module path. This is what makes the name ambiguous.
#[starknet::contract]
mod HelloStarknet {
    #[storage]
    struct Storage {
        counter: felt252,
    }
}

#[test]
fn declare_ambiguous_contract() {
    declare("HelloStarknet").unwrap();
}

#[test]
fn declare_by_module_path() {
    let _ = declare("declare_paths::HelloStarknet").unwrap().contract_class();
}
