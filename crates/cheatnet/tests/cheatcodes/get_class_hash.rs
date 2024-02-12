use crate::common::state::{build_runtime_state, create_cached_state};
use crate::common::{call_contract, deploy_wrapper};
use crate::common::{felt_selector_from_name, get_contracts};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::get_class_hash::get_class_hash;
use cheatnet::state::CheatnetState;
use conversions::felt252::FromShortString;
use conversions::IntoConv;

#[test]
fn get_class_hash_simple() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();
    let contract_name = Felt252::from_short_string("HelloStarknet").unwrap();
    let class_hash = declare(&mut cached_state, &contract_name, &contracts).unwrap();
    let contract_address =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    assert_eq!(
        class_hash,
        get_class_hash(&mut cached_state, contract_address).unwrap()
    );
}

#[test]
fn get_class_hash_upgrade() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();
    let contract_name = Felt252::from_short_string("GetClassHashCheckerUpg").unwrap();
    let class_hash = declare(&mut cached_state, &contract_name, &contracts).unwrap();
    let contract_address =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    assert_eq!(
        class_hash,
        get_class_hash(&mut cached_state, contract_address).unwrap()
    );

    let contract_name = Felt252::from_short_string("HelloStarknet").unwrap();
    let hello_starknet_class_hash = declare(&mut cached_state, &contract_name, &contracts).unwrap();

    let selector = felt_selector_from_name("upgrade");
    call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[hello_starknet_class_hash.into_()],
    );

    assert_eq!(
        hello_starknet_class_hash,
        get_class_hash(&mut cached_state, contract_address).unwrap()
    );
}
