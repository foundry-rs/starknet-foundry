use crate::common::state::{create_cached_state, create_cheatnet_state};
use crate::common::{felt_selector_from_name, get_contracts};
use blockifier::abi::abi_utils::starknet_keccak;
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::call_contract;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::deploy;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::load::{
    calculate_variable_address, load,
};
use conversions::felt252::FromShortString;

#[test]
fn load_simple_state() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("HelloStarknet").unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let contract_address = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    let selector = felt_selector_from_name("increase_balance");

    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[Felt252::from(420)],
    )
    .unwrap();

    let balance_value = load(
        &mut blockifier_state,
        contract_address,
        &calculate_variable_address(felt_selector_from_name("balance"), None),
        &Felt252::from(1),
    )
    .unwrap();

    assert_eq!(balance_value.len(), 1, "Wrong data amount was returned");
    let returned_balance_value = balance_value[0].clone();
    assert_eq!(
        returned_balance_value,
        Felt252::from(420),
        "Wrong data value was returned: {returned_balance_value}"
    );
}

#[test]
fn load_state_map_simple_value() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("MapSimpleValueSimpleKey").unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let contract_address = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    let selector = felt_selector_from_name("insert");

    let map_key = Felt252::from(420);
    let inserted_value = Felt252::from(69);
    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[map_key.clone(), inserted_value.clone()],
    )
    .unwrap();

    let var_selector = felt_selector_from_name("values");
    let var_address = calculate_variable_address(var_selector, Some(&[map_key]));

    let map_value = load(
        &mut blockifier_state,
        contract_address,
        &var_address,
        &Felt252::from(1),
    )
    .unwrap();

    assert_eq!(map_value.len(), 1, "Wrong data amount was returned");
    let returned_map_value = map_value[0].clone();
    assert_eq!(
        returned_map_value, inserted_value,
        "Wrong data value was returned: {returned_map_value}"
    );
}

#[test]
fn load_state_map_complex_value() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("MapComplexValueSimpleKey").unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let contract_address = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    let selector = felt_selector_from_name("insert");

    let map_key = Felt252::from(420);
    let inserted_values = vec![Felt252::from(68), Felt252::from(69)];
    let mut calldata = vec![map_key.clone()];
    calldata.append(&mut inserted_values.clone());
    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &calldata,
    )
    .unwrap();

    let var_selector = felt_selector_from_name("values");
    let variable_address = calculate_variable_address(var_selector, Some(&[map_key]));

    let map_value = load(
        &mut blockifier_state,
        contract_address,
        &variable_address,
        &Felt252::from(2),
    )
    .unwrap();

    assert_eq!(map_value.len(), 2, "Wrong data amount was returned");

    assert_eq!(
        map_value, inserted_values,
        "Wrong data value was returned: {map_value:?}"
    );
}

#[test]
fn load_state_map_complex_key() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("MapSimpleValueComplexKey").unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let contract_address = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    let selector = felt_selector_from_name("insert");

    let map_key = vec![Felt252::from(68), Felt252::from(69)];
    let mut calldata = map_key.clone();
    let inserted_value = Felt252::from(420);
    calldata.push(inserted_value.clone());
    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &calldata,
    )
    .unwrap();

    let var_selector = felt_selector_from_name("values");
    let var_address = calculate_variable_address(var_selector, Some(&map_key));

    let map_value = load(
        &mut blockifier_state,
        contract_address,
        &var_address,
        &Felt252::from(1),
    )
    .unwrap();

    assert_eq!(map_value.len(), 1, "Wrong data amount was returned");
    let returned_map_value = map_value[0].clone();
    assert_eq!(
        inserted_value, returned_map_value,
        "Wrong data value was returned: {returned_map_value}"
    );
}

#[test]
fn load_state_struct() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("FlatStateStruct").unwrap();
    let contracts = get_contracts();

    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let contract_address = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    let selector = felt_selector_from_name("insert");

    let calldata = vec![Felt252::from(68), Felt252::from(69)];
    call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &calldata,
    )
    .unwrap();

    let variable_name_hashed = starknet_keccak("value".as_ref());
    let struct_value = load(
        &mut blockifier_state,
        contract_address,
        &variable_name_hashed,
        &Felt252::from(2),
    )
    .unwrap();

    assert_eq!(struct_value.len(), 2, "Wrong data amount was returned");

    assert_eq!(
        calldata, struct_value,
        "Wrong data value was returned: {struct_value:?}"
    );
}
