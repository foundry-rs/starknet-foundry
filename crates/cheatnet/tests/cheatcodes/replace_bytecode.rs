use crate::{assert_success, cheatcodes::test_environment::TestEnvironment, common::get_contracts};
use cairo_felt::Felt252;
use cheatnet::state::CheatnetState;

#[test]
fn override_entrypoint() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);
    let contracts = get_contracts();

    let class_hash_a = test_env.declare("ReplaceBytecodeA", &contracts);
    let class_hash_b = test_env.declare("ReplaceBytecodeB", &contracts);
    let contract_address = test_env.deploy_wrapper(&class_hash_a, &[]);

    let output = test_env.call_contract(&contract_address, "get_const", &[]);

    assert_success!(output, vec![Felt252::from(2137)]);

    test_env
        .cheatnet_state()
        .replace_class_for_contract(contract_address, class_hash_b);

    let output = test_env.call_contract(&contract_address, "get_const", &[]);

    assert_success!(output, vec![Felt252::from(420)]);
}

#[test]
fn keep_storage() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);
    let contracts = get_contracts();

    let class_hash_a = test_env.declare("ReplaceBytecodeA", &contracts);
    let class_hash_b = test_env.declare("ReplaceBytecodeB", &contracts);
    let contract_address = test_env.deploy_wrapper(&class_hash_a, &[]);

    let output = test_env.call_contract(&contract_address, "set", &[456.into()]);

    assert_success!(output, vec![]);

    let output = test_env.call_contract(&contract_address, "get", &[]);

    assert_success!(output, vec![Felt252::from(456)]);

    test_env
        .cheatnet_state()
        .replace_class_for_contract(contract_address, class_hash_b);

    let output = test_env.call_contract(&contract_address, "get", &[]);

    assert_success!(output, vec![Felt252::from(556)]);
}

#[test]
fn allow_setting_original_class() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);
    let contracts = get_contracts();

    let class_hash_a = test_env.declare("ReplaceBytecodeA", &contracts);
    let class_hash_b = test_env.declare("ReplaceBytecodeB", &contracts);
    let contract_address = test_env.deploy_wrapper(&class_hash_a, &[]);

    test_env
        .cheatnet_state()
        .replace_class_for_contract(contract_address, class_hash_b);

    test_env
        .cheatnet_state()
        .replace_class_for_contract(contract_address, class_hash_a);

    let output = test_env.call_contract(&contract_address, "get_const", &[]);

    assert_success!(output, vec![Felt252::from(2137)]);
}
