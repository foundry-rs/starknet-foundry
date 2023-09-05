use crate::common::get_contracts;
use crate::common::state::create_cheatnet_state;
use cairo_felt::Felt252;
use cheatnet::cheatcodes::{CheatcodeError, EnhancedHintError};
use cheatnet::conversions::{class_hash_from_felt, felt_from_short_string};

#[test]
fn declare_simple() {
    let mut state = create_cheatnet_state();

    let contract = felt_from_short_string("HelloStarknet");
    let contracts = get_contracts();

    let class_hash = state.declare(&contract, &contracts).unwrap();

    assert_ne!(class_hash, class_hash_from_felt(&Felt252::from(0)));
}

#[test]
fn declare_multiple() {
    let mut state = create_cheatnet_state();

    let contract = felt_from_short_string("HelloStarknet");
    let contracts = get_contracts();

    let class_hash = state.declare(&contract, &contracts).unwrap();

    let contract = felt_from_short_string("ConstructorSimple");

    let class_hash2 = state.declare(&contract, &contracts).unwrap();

    assert_ne!(class_hash, class_hash_from_felt(&Felt252::from(0)));
    assert_ne!(class_hash2, class_hash_from_felt(&Felt252::from(0)));
    assert_ne!(class_hash, class_hash2);
}

#[test]
fn declare_same_contract() {
    let mut state = create_cheatnet_state();

    let contract = felt_from_short_string("HelloStarknet");
    let contracts = get_contracts();

    let class_hash = state.declare(&contract, &contracts).unwrap();
    assert_ne!(class_hash, class_hash_from_felt(&Felt252::from(0)));

    let contract = felt_from_short_string("HelloStarknet");

    let output = state.declare(&contract, &contracts);

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Anyhow(msg))) => {
            msg.to_string().contains("is already declared")
        }
        _ => false,
    });
}

#[test]
fn declare_non_existant() {
    let mut state = create_cheatnet_state();

    let contract = felt_from_short_string("GoodbyeStarknet");
    let contracts = get_contracts();

    let output = state.declare(&contract, &contracts);

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Anyhow(msg))) => {
            msg.to_string().contains("Failed") && msg.to_string().contains("GoodbyeStarknet")
        }
        _ => false,
    });
}
