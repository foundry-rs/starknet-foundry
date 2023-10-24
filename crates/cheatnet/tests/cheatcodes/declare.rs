use crate::common::state::create_cheatnet_state;
use crate::common::{get_contracts, state::create_cached_state};
use cheatnet::cheatcodes::{CheatcodeError, EnhancedHintError};
use conversions::StarknetConversions;
use starknet_api::core::ClassHash;
use starknet_api::hash::StarkFelt;
use starknet_api::stark_felt;

static HELLO_STARKNET_CLASS_HASH: &str =
    "0x298f80e468953d1e65314b6bc63347c7a3fe454a89c2b15387dd52ee281d103";
static CONSTRUCTOR_SIMPLE_CLASS_HASH: &str =
    "0x02dbeae7583f3dd4af0bc2da4d58611433165fec7e31245bfa2f1378fbff6a4c";

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
