use crate::assert_success;
use crate::cheatcodes::{map_entry_address, variable_address};
use crate::common::call_contract;
use crate::common::state::{create_cached_state, create_cheatnet_state};
use crate::common::{felt_selector_from_name, get_contracts};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::deploy;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::store;
use conversions::felt252::FromShortString;

#[test]
fn store_simple_state() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("HelloStarknet").unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let contract_address = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    store(
        &mut blockifier_state,
        contract_address,
        &variable_address("balance"),
        Felt252::from(666),
    )
    .unwrap();

    let selector = felt_selector_from_name("get_balance");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(666)]);
}

#[test]
fn store_state_map_simple_value() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("MapSimpleValueSimpleKey").unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let contract_address = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    let map_key = Felt252::from(420);
    let inserted_value = Felt252::from(69);

    let entry_address = map_entry_address("values", &[map_key.clone()]);
    store(
        &mut blockifier_state,
        contract_address,
        &entry_address,
        inserted_value.clone(),
    )
    .unwrap();

    let selector = felt_selector_from_name("read");
    let call_output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[map_key],
    )
    .unwrap();

    assert_success!(call_output, vec![inserted_value]);
}
