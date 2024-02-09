use crate::cheatcodes::{map_entry_address, variable_address};
use crate::common::state::{build_runtime_state, create_cached_state};
use crate::common::{call_contract, deploy_wrapper};
use crate::common::{felt_selector_from_name, get_contracts};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::load;
use cheatnet::state::CheatnetState;
use conversions::felt252::FromShortString;

#[test]
fn load_simple_state() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract = Felt252::from_short_string("HelloStarknet").unwrap();
    let contracts = get_contracts();

    let class_hash = declare(&mut cached_state, &contract, &contracts).unwrap();

    let contract_address =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    let selector = felt_selector_from_name("increase_balance");

    call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[Felt252::from(420)],
    );

    let balance_value = load(
        &mut cached_state,
        contract_address,
        &variable_address("balance"),
    )
    .unwrap();

    assert_eq!(
        balance_value,
        Felt252::from(420),
        "Wrong data value was returned: {balance_value}"
    );
}

#[test]
fn load_state_map_simple_value() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract = Felt252::from_short_string("MapSimpleValueSimpleKey").unwrap();
    let contracts = get_contracts();

    let class_hash = declare(&mut cached_state, &contract, &contracts).unwrap();

    let contract_address =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    let selector = felt_selector_from_name("insert");

    let map_key = Felt252::from(420);
    let inserted_value = Felt252::from(69);
    call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[map_key.clone(), inserted_value.clone()],
    );

    let var_address = map_entry_address("values", &[map_key]);
    let map_value = load(&mut cached_state, contract_address, &var_address).unwrap();

    assert_eq!(
        map_value, inserted_value,
        "Wrong data value was returned: {map_value}"
    );
}
