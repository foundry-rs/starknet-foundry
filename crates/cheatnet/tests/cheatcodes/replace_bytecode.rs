use crate::assert_success;
use crate::common::state::{build_runtime_state, create_cached_state};
use crate::common::{call_contract, deploy_wrapper, felt_selector_from_name, get_contracts};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::state::CheatnetState;

#[test]
fn override_entrypoint() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();

    let class_hash_a = declare(&mut cached_state, "ReplaceBytecodeA", &contracts).unwrap();
    let class_hash_b = declare(&mut cached_state, "ReplaceBytecodeB", &contracts).unwrap();
    let contract_address =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash_a, &[]).unwrap();

    let selector = felt_selector_from_name("get_const");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(2137)]);

    runtime_state
        .cheatnet_state
        .replace_class_for_contract(contract_address, class_hash_b);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(420)]);
}

#[test]
fn keep_storage() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();

    let class_hash_a = declare(&mut cached_state, "ReplaceBytecodeA", &contracts).unwrap();
    let class_hash_b = declare(&mut cached_state, "ReplaceBytecodeB", &contracts).unwrap();
    let contract_address =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash_a, &[]).unwrap();

    let set_selector = felt_selector_from_name("set");
    let get_selector = felt_selector_from_name("get");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &set_selector,
        &[456.into()],
    );

    assert_success!(output, vec![]);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &get_selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(456)]);

    runtime_state
        .cheatnet_state
        .replace_class_for_contract(contract_address, class_hash_b);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &get_selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(556)]);
}

#[test]
fn allow_setting_original_class() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();

    let class_hash_a = declare(&mut cached_state, "ReplaceBytecodeA", &contracts).unwrap();
    let class_hash_b = declare(&mut cached_state, "ReplaceBytecodeB", &contracts).unwrap();
    let contract_address =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash_a, &[]).unwrap();

    runtime_state
        .cheatnet_state
        .replace_class_for_contract(contract_address, class_hash_b);
    runtime_state
        .cheatnet_state
        .replace_class_for_contract(contract_address, class_hash_a);

    let selector = felt_selector_from_name("get_const");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(2137)]);
}
