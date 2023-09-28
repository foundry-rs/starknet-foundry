use crate::common::get_contracts;
use crate::common::state::create_cheatnet_state;
use cairo_felt::Felt252;
use cheatnet::cheatcodes::{CheatcodeError, EnhancedHintError};
use conversions::StarknetConversions;

#[test]
fn declare_simple() {
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state();

    let contract = "HelloStarknet".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    assert_ne!(class_hash, Felt252::from(0).to_class_hash());
}

#[test]
fn declare_multiple() {
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state();

    let contract = "HelloStarknet".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let contract = "ConstructorSimple".to_owned().to_felt252();

    let class_hash2 = blockifier_state.declare(&contract, &contracts).unwrap();

    assert_ne!(class_hash, Felt252::from(0).to_class_hash());
    assert_ne!(class_hash2, Felt252::from(0).to_class_hash());
    assert_ne!(class_hash, class_hash2);
}

#[test]
fn declare_same_contract() {
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state();

    let contract = "HelloStarknet".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    assert_ne!(class_hash, Felt252::from(0).to_class_hash());

    let contract = "HelloStarknet".to_owned().to_felt252();

    let output = blockifier_state.declare(&contract, &contracts);

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Anyhow(msg))) => {
            msg.to_string().contains("is already declared")
        }
        _ => false,
    });
}

#[test]
fn declare_non_existant() {
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state();

    let contract = "GoodbyeStarknet".to_owned().to_felt252();
    let contracts = get_contracts();

    let output = blockifier_state.declare(&contract, &contracts);

    assert!(match output {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::Anyhow(msg))) => {
            msg.to_string().contains("Failed") && msg.to_string().contains("GoodbyeStarknet")
        }
        _ => false,
    });
}
