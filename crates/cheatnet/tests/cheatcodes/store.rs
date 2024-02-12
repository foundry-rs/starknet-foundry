use crate::assert_success;
use crate::cheatcodes::{map_entry_address, variable_address};
use crate::common::state::{build_runtime_state, create_cached_state};
use crate::common::{call_contract, deploy_wrapper};
use crate::common::{felt_selector_from_name, get_contracts};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::store;
use cheatnet::state::CheatnetState;
use conversions::felt252::FromShortString;

#[test]
fn store_simple_state() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract = Felt252::from_short_string("HelloStarknet").unwrap();
    let contracts = get_contracts();

    let class_hash = declare(&mut cached_state, &contract, &contracts).unwrap();

    let contract_address =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    store(
        &mut cached_state,
        contract_address,
        &variable_address("balance"),
        Felt252::from(666),
    )
    .unwrap();

    let selector = felt_selector_from_name("get_balance");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(666)]);
}

#[test]
fn store_state_map_simple_value() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract = Felt252::from_short_string("MapSimpleValueSimpleKey").unwrap();
    let contracts = get_contracts();

    let class_hash = declare(&mut cached_state, &contract, &contracts).unwrap();

    let contract_address =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    let map_key = Felt252::from(420);
    let inserted_value = Felt252::from(69);

    let entry_address = map_entry_address("values", &[map_key.clone()]);
    store(
        &mut cached_state,
        contract_address,
        &entry_address,
        inserted_value.clone(),
    )
    .unwrap();

    let selector = felt_selector_from_name("read");
    let call_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[map_key],
    );

    assert_success!(call_output, vec![inserted_value]);
}
