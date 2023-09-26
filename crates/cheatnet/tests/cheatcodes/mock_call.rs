use crate::common::{felt_selector_from_name, recover_data};
use crate::{
    assert_success,
    common::{deploy_contract, get_contracts, state::create_cheatnet_state},
};
use cairo_felt::Felt252;
use cheatnet::rpc::call_contract;
use conversions::StarknetConversions;
use starknet_api::core::ContractAddress;

#[test]
fn mock_call_simple() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "MockChecker", &[Felt252::from(420)]);

    let selector = felt_selector_from_name("get_thing");
    let ret_data = vec![Felt252::from(123)];

    state.start_mock_call(
        contract_address,
        &"get_thing".to_owned().to_felt252(),
        &ret_data,
    );

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, ret_data);
}

#[test]
fn mock_call_stop() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "MockChecker", &[Felt252::from(420)]);

    let selector = felt_selector_from_name("get_thing");
    let ret_data = vec![Felt252::from(123)];

    state.start_mock_call(
        contract_address,
        &"get_thing".to_owned().to_felt252(),
        &ret_data,
    );

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, ret_data);

    state.stop_mock_call(contract_address, &"get_thing".to_owned().to_felt252());

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(420)]);
}

#[test]
fn mock_call_stop_no_start() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "MockChecker", &[Felt252::from(420)]);

    let selector = felt_selector_from_name("get_thing");

    state.stop_mock_call(contract_address, &"get_thing".to_owned().to_felt252());

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(420)]);
}

#[test]
fn mock_call_double() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "MockChecker", &[Felt252::from(420)]);

    let selector = felt_selector_from_name("get_thing");

    let ret_data = vec![Felt252::from(123)];
    state.start_mock_call(
        contract_address,
        &"get_thing".to_owned().to_felt252(),
        &ret_data,
    );

    let ret_data = vec![Felt252::from(999)];
    state.start_mock_call(
        contract_address,
        &"get_thing".to_owned().to_felt252(),
        &ret_data,
    );

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, ret_data);

    state.stop_mock_call(contract_address, &"get_thing".to_owned().to_felt252());

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(420)]);
}

#[test]
fn mock_call_double_call() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "MockChecker", &[Felt252::from(420)]);

    let selector = felt_selector_from_name("get_thing");

    let ret_data = vec![Felt252::from(123)];
    state.start_mock_call(
        contract_address,
        &"get_thing".to_owned().to_felt252(),
        &ret_data,
    );

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, ret_data);

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, ret_data);
}

#[test]
fn mock_call_proxy() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "MockChecker", &[Felt252::from(420)]);
    let selector = felt_selector_from_name("get_thing");

    let ret_data = vec![Felt252::from(123)];
    state.start_mock_call(
        contract_address,
        &"get_thing".to_owned().to_felt252(),
        &ret_data,
    );

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, ret_data);

    let proxy_address = deploy_contract(&mut state, "MockCheckerProxy", &[]);
    let proxy_selector = felt_selector_from_name("get_thing_from_contract");
    let output = call_contract(
        &proxy_address,
        &proxy_selector,
        &[contract_address.to_felt252()],
        &mut state,
    )
    .unwrap();

    assert_success!(output, ret_data);
}

#[test]
fn mock_call_proxy_with_other_syscall() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "MockChecker", &[Felt252::from(420)]);
    let selector = felt_selector_from_name("get_thing");

    let ret_data = vec![Felt252::from(123)];
    state.start_mock_call(
        contract_address,
        &"get_thing".to_owned().to_felt252(),
        &ret_data,
    );

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, ret_data);

    let proxy_address = deploy_contract(&mut state, "MockCheckerProxy", &[]);
    let proxy_selector = felt_selector_from_name("get_thing_from_contract_and_emit_event");
    let output = call_contract(
        &proxy_address,
        &proxy_selector,
        &[contract_address.to_felt252()],
        &mut state,
    )
    .unwrap();

    assert_success!(output, ret_data);
}

#[test]
fn mock_call_inner_call_no_effect() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "MockChecker", &[Felt252::from(420)]);

    let selector = felt_selector_from_name("get_thing");
    let ret_data = vec![Felt252::from(123)];

    state.start_mock_call(
        contract_address,
        &"get_thing".to_owned().to_felt252(),
        &ret_data,
    );

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, ret_data);

    let selector = felt_selector_from_name("get_thing_wrapper");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(420)]);
}

#[test]
fn mock_call_library_call_no_effect() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();
    let contract_name = "MockChecker".to_owned().to_felt252();
    let class_hash = state.declare(&contract_name, &contracts).unwrap();

    let contract_address = state
        .deploy(&class_hash, &[Felt252::from(420)])
        .unwrap()
        .contract_address;

    let lib_call_address = deploy_contract(&mut state, "MockCheckerLibCall", &[]);

    let ret_data = vec![Felt252::from(123)];
    state.start_mock_call(
        contract_address,
        &"get_constant_thing".to_owned().to_felt252(),
        &ret_data,
    );

    let lib_call_selector = felt_selector_from_name("get_constant_thing_with_lib_call");
    let output = call_contract(
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.to_felt252()],
        &mut state,
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(13)]);
}

#[test]
fn mock_call_before_deployment() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();
    let contract_name = "MockChecker".to_owned().to_felt252();
    let class_hash = state.declare(&contract_name, &contracts).unwrap();

    let precalculated_address = state.precalculate_address(&class_hash, &[Felt252::from(420)]);

    let selector = felt_selector_from_name("get_thing");
    let ret_data = vec![Felt252::from(123)];
    state.start_mock_call(
        precalculated_address,
        &"get_thing".to_owned().to_felt252(),
        &ret_data,
    );

    let contract_address = state
        .deploy(&class_hash, &[Felt252::from(420)])
        .unwrap()
        .contract_address;

    assert_eq!(precalculated_address, contract_address);

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, ret_data);
}

#[test]
fn mock_call_not_implemented() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "MockChecker", &[Felt252::from(420)]);

    let selector = felt_selector_from_name("get_thing_not_implemented");
    let ret_data = vec![Felt252::from(123), Felt252::from(123), Felt252::from(123)];

    state.start_mock_call(
        contract_address,
        &"get_thing_not_implemented".to_owned().to_felt252(),
        &ret_data,
    );

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, ret_data);
}

#[test]
fn mock_call_in_constructor() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();

    let contract_name = "HelloStarknet".to_owned().to_felt252();
    let class_hash = state.declare(&contract_name, &contracts).unwrap();
    let balance_contract_address = state.deploy(&class_hash, &[]).unwrap().contract_address;
    let ret_data = vec![Felt252::from(223)];
    state.start_mock_call(
        balance_contract_address,
        &"get_balance".to_owned().to_felt252(),
        &ret_data,
    );

    let contract_name = "ConstructorMockChecker".to_owned().to_felt252();
    let class_hash = state.declare(&contract_name, &contracts).unwrap();
    let contract_address = state
        .deploy(&class_hash, &[balance_contract_address.to_felt252()])
        .unwrap()
        .contract_address;

    let selector = felt_selector_from_name("get_constructor_balance");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();
    let output_data = recover_data(output);
    assert_eq!(output_data.len(), 1);
    assert_eq!(output_data.get(0).unwrap().clone(), Felt252::from(223));
}

#[test]
fn mock_call_two_methods() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "MockChecker", &[Felt252::from(420)]);

    let selector1 = felt_selector_from_name("get_thing");
    let selector2 = felt_selector_from_name("get_constant_thing");

    let ret_data = vec![Felt252::from(123)];
    state.start_mock_call(
        contract_address,
        &"get_thing".to_owned().to_felt252(),
        &ret_data,
    );

    state.start_mock_call(
        contract_address,
        &"get_constant_thing".to_owned().to_felt252(),
        &ret_data,
    );

    let output = call_contract(&contract_address, &selector1, &[], &mut state).unwrap();

    assert_success!(output, ret_data);

    let output = call_contract(&contract_address, &selector2, &[], &mut state).unwrap();

    assert_success!(output, ret_data);
}

#[test]
fn mock_call_nonexisting_contract() {
    let mut state = create_cheatnet_state();

    let selector = felt_selector_from_name("get_thing");
    let ret_data = vec![Felt252::from(123)];

    let contract_address = ContractAddress::from(218_u8);

    state.start_mock_call(
        contract_address,
        &"get_thing".to_owned().to_felt252(),
        &ret_data,
    );

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, ret_data);
}
