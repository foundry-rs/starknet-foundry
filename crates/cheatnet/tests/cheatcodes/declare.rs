use crate::common::state::create_cheatnet_state;
use crate::common::{get_contracts, state::create_cached_state};
use cheatnet::cheatcodes::{CheatcodeError, EnhancedHintError};
use conversions::StarknetConversions;
use starknet_api::core::ClassHash;
use starknet_api::hash::StarkFelt;
use starknet_api::stark_felt;

static HELLO_STARKNET_CLASS_HASH: &str =
    "0x029c29c28d7eed24e3e57e6189cda4fc5a7a5f1d097f02c6692cff91816a5fbc";
static CONSTRUCTOR_SIMPLE_CLASS_HASH: &str =
    "0x045b3e15b898999ca953f197065fb4b4f98af6e92fb012bb26917a8020c3314d";

#[test]
fn declare_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, _) = create_cheatnet_state(&mut cached_state);

    let contract = "HelloStarknet".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    assert_eq!(
        class_hash,
        ClassHash(stark_felt!(HELLO_STARKNET_CLASS_HASH))
    );
}

#[test]
fn declare_multiple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, _) = create_cheatnet_state(&mut cached_state);

    let contract = "HelloStarknet".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let contract = "ConstructorSimple".to_owned().to_felt252();

    let class_hash2 = blockifier_state.declare(&contract, &contracts).unwrap();

    assert_eq!(
        class_hash,
        ClassHash(stark_felt!(HELLO_STARKNET_CLASS_HASH))
    );
    assert_eq!(
        class_hash2,
        ClassHash(stark_felt!(CONSTRUCTOR_SIMPLE_CLASS_HASH))
    );
}

#[test]
fn declare_same_contract() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, _) = create_cheatnet_state(&mut cached_state);

    let contract = "HelloStarknet".to_owned().to_felt252();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();
    assert_eq!(
        class_hash,
        ClassHash(stark_felt!(HELLO_STARKNET_CLASS_HASH))
    );

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
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, _) = create_cheatnet_state(&mut cached_state);

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
