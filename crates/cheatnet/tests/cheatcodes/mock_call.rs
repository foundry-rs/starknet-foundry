use crate::common::felt_selector_from_name;
use crate::common::state::{build_runtime_state, create_cached_state};
use crate::common::{call_contract, deploy_wrapper};
use crate::{
    assert_success,
    common::{deploy_contract, get_contracts},
};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::state::CheatnetState;
use conversions::felt252::FromShortString;
use conversions::IntoConv;
use starknet_api::core::ContractAddress;

use super::test_environment::TestEnvironment;

trait MockCallTrait {
    fn start_mock_call(
        &mut self,
        contract_address: &ContractAddress,
        function_name: &str,
        ret_data: &[u128],
    );
    fn stop_mock_call(&mut self, contract_address: &ContractAddress, function_name: &str);
}

impl<'a> MockCallTrait for TestEnvironment<'a> {
    fn start_mock_call(
        &mut self,
        contract_address: &ContractAddress,
        function_name: &str,
        ret_data: &[u128],
    ) {
        let function_name = Felt252::from_short_string(function_name).unwrap();
        let ret_data: Vec<Felt252> = ret_data.iter().map(|x| Felt252::from(*x)).collect();
        self.runtime_state.cheatnet_state.start_mock_call(
            *contract_address,
            &function_name,
            &ret_data,
        );
    }

    fn stop_mock_call(&mut self, contract_address: &ContractAddress, function_name: &str) {
        let function_name = Felt252::from_short_string(function_name).unwrap();
        self.runtime_state
            .cheatnet_state
            .stop_mock_call(*contract_address, &function_name);
    }
}

#[test]
fn mock_call_simple() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "MockChecker",
        &[Felt252::from(420)],
    );

    let selector = felt_selector_from_name("get_thing");
    let ret_data = vec![Felt252::from(123)];

    runtime_state.cheatnet_state.start_mock_call(
        contract_address,
        &Felt252::from_short_string("get_thing").unwrap(),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, ret_data);
}

#[test]
fn mock_call_stop() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "MockChecker",
        &[Felt252::from(420)],
    );

    let selector = felt_selector_from_name("get_thing");
    let ret_data = vec![Felt252::from(123)];

    runtime_state.cheatnet_state.start_mock_call(
        contract_address,
        &Felt252::from_short_string("get_thing").unwrap(),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, ret_data);

    runtime_state.cheatnet_state.stop_mock_call(
        contract_address,
        &Felt252::from_short_string("get_thing").unwrap(),
    );

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
fn mock_call_stop_no_start() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "MockChecker",
        &[Felt252::from(420)],
    );

    let selector = felt_selector_from_name("get_thing");

    runtime_state.cheatnet_state.stop_mock_call(
        contract_address,
        &Felt252::from_short_string("get_thing").unwrap(),
    );

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
fn mock_call_double() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "MockChecker",
        &[Felt252::from(420)],
    );

    let selector = felt_selector_from_name("get_thing");

    let ret_data = vec![Felt252::from(123)];
    runtime_state.cheatnet_state.start_mock_call(
        contract_address,
        &Felt252::from_short_string("get_thing").unwrap(),
        &ret_data,
    );

    let ret_data = vec![Felt252::from(999)];
    runtime_state.cheatnet_state.start_mock_call(
        contract_address,
        &Felt252::from_short_string("get_thing").unwrap(),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, ret_data);

    runtime_state.cheatnet_state.stop_mock_call(
        contract_address,
        &Felt252::from_short_string("get_thing").unwrap(),
    );

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
fn mock_call_double_call() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "MockChecker",
        &[Felt252::from(420)],
    );

    let selector = felt_selector_from_name("get_thing");

    let ret_data = vec![Felt252::from(123)];
    runtime_state.cheatnet_state.start_mock_call(
        contract_address,
        &Felt252::from_short_string("get_thing").unwrap(),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, ret_data);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, ret_data);
}

#[test]
fn mock_call_proxy() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "MockChecker",
        &[Felt252::from(420)],
    );
    let selector = felt_selector_from_name("get_thing");

    let ret_data = vec![Felt252::from(123)];
    runtime_state.cheatnet_state.start_mock_call(
        contract_address,
        &Felt252::from_short_string("get_thing").unwrap(),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, ret_data);

    let proxy_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "MockCheckerProxy",
        &[],
    );
    let proxy_selector = felt_selector_from_name("get_thing_from_contract");
    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    );

    assert_success!(output, ret_data);
}

#[test]
fn mock_call_proxy_with_other_syscall() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "MockChecker",
        &[Felt252::from(420)],
    );
    let selector = felt_selector_from_name("get_thing");

    let ret_data = vec![Felt252::from(123)];
    runtime_state.cheatnet_state.start_mock_call(
        contract_address,
        &Felt252::from_short_string("get_thing").unwrap(),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, ret_data);

    let proxy_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "MockCheckerProxy",
        &[],
    );
    let proxy_selector = felt_selector_from_name("get_thing_from_contract_and_emit_event");
    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    );

    assert_success!(output, ret_data);
}

#[test]
fn mock_call_inner_call_no_effect() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "MockChecker",
        &[Felt252::from(420)],
    );

    let selector = felt_selector_from_name("get_thing");
    let ret_data = vec![Felt252::from(123)];

    runtime_state.cheatnet_state.start_mock_call(
        contract_address,
        &Felt252::from_short_string("get_thing").unwrap(),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, ret_data);

    let selector = felt_selector_from_name("get_thing_wrapper");

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
fn mock_call_library_call_no_effect() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = declare(&mut cached_state, "MockChecker", &contracts).unwrap();

    let contract_address = deploy_wrapper(
        &mut cached_state,
        &mut runtime_state,
        &class_hash,
        &[Felt252::from(420)],
    )
    .unwrap();

    let lib_call_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "MockCheckerLibCall",
        &[],
    );

    let ret_data = vec![Felt252::from(123)];
    runtime_state.cheatnet_state.start_mock_call(
        contract_address,
        &Felt252::from_short_string("get_constant_thing").unwrap(),
        &ret_data,
    );

    let lib_call_selector = felt_selector_from_name("get_constant_thing_with_lib_call");
    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    );

    assert_success!(output, vec![Felt252::from(13)]);
}

#[test]
fn mock_call_before_deployment() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = declare(&mut cached_state, "MockChecker", &contracts).unwrap();

    let precalculated_address = runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &[Felt252::from(420)]);

    let selector = felt_selector_from_name("get_thing");
    let ret_data = vec![Felt252::from(123)];
    runtime_state.cheatnet_state.start_mock_call(
        precalculated_address,
        &Felt252::from_short_string("get_thing").unwrap(),
        &ret_data,
    );

    let contract_address = deploy_wrapper(
        &mut cached_state,
        &mut runtime_state,
        &class_hash,
        &[Felt252::from(420)],
    )
    .unwrap();

    assert_eq!(precalculated_address, contract_address);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, ret_data);
}

#[test]
fn mock_call_not_implemented() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "MockChecker",
        &[Felt252::from(420)],
    );

    let selector = felt_selector_from_name("get_thing_not_implemented");
    let ret_data = vec![Felt252::from(123), Felt252::from(123), Felt252::from(123)];

    runtime_state.cheatnet_state.start_mock_call(
        contract_address,
        &Felt252::from_short_string("get_thing_not_implemented").unwrap(),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, ret_data);
}

#[test]
fn mock_call_in_constructor() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();

    let class_hash = declare(&mut cached_state, "HelloStarknet", &contracts).unwrap();
    let balance_contract_address =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();
    let ret_data = vec![Felt252::from(223)];
    runtime_state.cheatnet_state.start_mock_call(
        balance_contract_address,
        &Felt252::from_short_string("get_balance").unwrap(),
        &ret_data,
    );

    let class_hash = declare(&mut cached_state, "ConstructorMockChecker", &contracts).unwrap();
    let contract_address = deploy_wrapper(
        &mut cached_state,
        &mut runtime_state,
        &class_hash,
        &[balance_contract_address.into_()],
    )
    .unwrap();

    let selector = felt_selector_from_name("get_constructor_balance");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    let output_data = output.recover_data();
    assert_eq!(output_data.len(), 1);
    assert_eq!(output_data.first().unwrap().clone(), Felt252::from(223));
}

#[test]
fn mock_call_two_methods() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "MockChecker",
        &[Felt252::from(420)],
    );

    let selector1 = felt_selector_from_name("get_thing");
    let selector2 = felt_selector_from_name("get_constant_thing");

    let ret_data = vec![Felt252::from(123)];
    runtime_state.cheatnet_state.start_mock_call(
        contract_address,
        &Felt252::from_short_string("get_thing").unwrap(),
        &ret_data,
    );

    runtime_state.cheatnet_state.start_mock_call(
        contract_address,
        &Felt252::from_short_string("get_constant_thing").unwrap(),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector1,
        &[],
    );

    assert_success!(output, ret_data);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector2,
        &[],
    );

    assert_success!(output, ret_data);
}

#[test]
fn mock_call_nonexisting_contract() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let selector = felt_selector_from_name("get_thing");
    let ret_data = vec![Felt252::from(123)];

    let contract_address = ContractAddress::from(218_u8);

    runtime_state.cheatnet_state.start_mock_call(
        contract_address,
        &Felt252::from_short_string("get_thing").unwrap(),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, ret_data);
}
