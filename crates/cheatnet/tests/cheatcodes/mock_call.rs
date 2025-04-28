use super::test_environment::TestEnvironment;
use crate::common::assertions::ClassHashAssert;
use crate::common::recover_data;
use crate::common::state::create_cached_state;
use crate::common::{call_contract, deploy_wrapper};
use crate::{
    common::assertions::assert_success,
    common::{deploy_contract, get_contracts},
};
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::felt_selector_from_name;
use cheatnet::state::{CheatSpan, CheatnetState};
use conversions::IntoConv;
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

trait MockCallTrait {
    fn mock_call(
        &mut self,
        contract_address: &ContractAddress,
        function_name: &str,
        ret_data: &[u128],
        span: CheatSpan,
    );
    fn stop_mock_call(&mut self, contract_address: &ContractAddress, function_name: &str);
}

impl MockCallTrait for TestEnvironment {
    fn mock_call(
        &mut self,
        contract_address: &ContractAddress,
        function_name: &str,
        ret_data: &[u128],
        span: CheatSpan,
    ) {
        let ret_data: Vec<Felt> = ret_data.iter().map(|x| Felt::from(*x)).collect();
        let function_selector = get_selector_from_name(function_name).unwrap();
        self.cheatnet_state.mock_call(
            *contract_address,
            function_selector.into_(),
            &ret_data,
            span,
        );
    }

    fn stop_mock_call(&mut self, contract_address: &ContractAddress, function_name: &str) {
        let function_selector = get_selector_from_name(function_name).unwrap();
        self.cheatnet_state
            .stop_mock_call(*contract_address, function_selector.into_());
    }
}

#[test]
fn mock_call_simple() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "MockChecker",
        &[Felt::from(420)],
    );

    let selector = felt_selector_from_name("get_thing");
    let ret_data = [Felt::from(123)];

    cheatnet_state.start_mock_call(
        contract_address,
        felt_selector_from_name("get_thing"),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &ret_data);
}

#[test]
fn mock_call_stop() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "MockChecker",
        &[Felt::from(420)],
    );

    let selector = felt_selector_from_name("get_thing");
    let ret_data = [Felt::from(123)];

    cheatnet_state.start_mock_call(
        contract_address,
        felt_selector_from_name("get_thing"),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &ret_data);

    cheatnet_state.stop_mock_call(contract_address, felt_selector_from_name("get_thing"));

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &[Felt::from(420)]);
}

#[test]
fn mock_call_stop_no_start() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "MockChecker",
        &[Felt::from(420)],
    );

    let selector = felt_selector_from_name("get_thing");

    cheatnet_state.stop_mock_call(contract_address, felt_selector_from_name("get_thing"));

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &[Felt::from(420)]);
}

#[test]
fn mock_call_double() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "MockChecker",
        &[Felt::from(420)],
    );

    let selector = felt_selector_from_name("get_thing");

    let ret_data = [Felt::from(123)];
    cheatnet_state.start_mock_call(contract_address, selector, &ret_data);

    let ret_data = [Felt::from(999)];
    cheatnet_state.start_mock_call(contract_address, selector, &ret_data);

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &ret_data);

    cheatnet_state.stop_mock_call(contract_address, selector);

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &[Felt::from(420)]);
}

#[test]
fn mock_call_double_call() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "MockChecker",
        &[Felt::from(420)],
    );

    let selector = felt_selector_from_name("get_thing");

    let ret_data = [Felt::from(123)];
    cheatnet_state.start_mock_call(
        contract_address,
        felt_selector_from_name("get_thing"),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &ret_data);

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &ret_data);
}

#[test]
fn mock_call_proxy() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "MockChecker",
        &[Felt::from(420)],
    );
    let selector = felt_selector_from_name("get_thing");

    let ret_data = [Felt::from(123)];
    cheatnet_state.start_mock_call(
        contract_address,
        felt_selector_from_name("get_thing"),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &ret_data);

    let proxy_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "MockCheckerProxy",
        &[],
    );
    let proxy_selector = felt_selector_from_name("get_thing_from_contract");
    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &proxy_address,
        proxy_selector,
        &[contract_address.into_()],
    );

    assert_success(output, &ret_data);
}

#[test]
fn mock_call_proxy_with_other_syscall() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "MockChecker",
        &[Felt::from(420)],
    );
    let selector = felt_selector_from_name("get_thing");

    let ret_data = [Felt::from(123)];
    cheatnet_state.start_mock_call(
        contract_address,
        felt_selector_from_name("get_thing"),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &ret_data);

    let proxy_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "MockCheckerProxy",
        &[],
    );
    let proxy_selector = felt_selector_from_name("get_thing_from_contract_and_emit_event");
    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &proxy_address,
        proxy_selector,
        &[contract_address.into_()],
    );

    assert_success(output, &ret_data);
}

#[test]
fn mock_call_inner_call_no_effect() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "MockChecker",
        &[Felt::from(420)],
    );

    let selector = felt_selector_from_name("get_thing");
    let ret_data = [Felt::from(123)];

    cheatnet_state.start_mock_call(
        contract_address,
        felt_selector_from_name("get_thing"),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &ret_data);

    let selector = felt_selector_from_name("get_thing_wrapper");

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &[Felt::from(420)]);
}

#[test]
fn mock_call_library_call_no_effect() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contracts_data = get_contracts();
    let class_hash = declare(&mut cached_state, "MockChecker", &contracts_data)
        .unwrap()
        .unwrap_success();

    let contract_address = deploy_wrapper(
        &mut cached_state,
        &mut cheatnet_state,
        &class_hash,
        &[Felt::from(420)],
    )
    .unwrap();

    let lib_call_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "MockCheckerLibCall",
        &[],
    );

    let ret_data = [Felt::from(123)];
    cheatnet_state.start_mock_call(
        contract_address,
        felt_selector_from_name("get_constant_thing"),
        &ret_data,
    );

    let lib_call_selector = felt_selector_from_name("get_constant_thing_with_lib_call");
    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &lib_call_address,
        lib_call_selector,
        &[class_hash.into_()],
    );

    assert_success(output, &[Felt::from(13)]);
}

#[test]
fn mock_call_before_deployment() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contracts_data = get_contracts();
    let class_hash = declare(&mut cached_state, "MockChecker", &contracts_data)
        .unwrap()
        .unwrap_success();

    let precalculated_address =
        cheatnet_state.precalculate_address(&class_hash, &[Felt::from(420)]);

    let selector = felt_selector_from_name("get_thing");
    let ret_data = [Felt::from(123)];
    cheatnet_state.start_mock_call(
        precalculated_address,
        felt_selector_from_name("get_thing"),
        &ret_data,
    );

    let contract_address = deploy_wrapper(
        &mut cached_state,
        &mut cheatnet_state,
        &class_hash,
        &[Felt::from(420)],
    )
    .unwrap();

    assert_eq!(precalculated_address, contract_address);

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &ret_data);
}

#[test]
fn mock_call_not_implemented() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "MockChecker",
        &[Felt::from(420)],
    );

    let selector = felt_selector_from_name("get_thing_not_implemented");
    let ret_data = [Felt::from(123), Felt::from(123), Felt::from(123)];

    cheatnet_state.start_mock_call(
        contract_address,
        felt_selector_from_name("get_thing_not_implemented"),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &ret_data);
}

#[test]
fn mock_call_in_constructor() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contracts_data = get_contracts();

    let class_hash = declare(&mut cached_state, "HelloStarknet", &contracts_data)
        .unwrap()
        .unwrap_success();
    let balance_contract_address =
        deploy_wrapper(&mut cached_state, &mut cheatnet_state, &class_hash, &[]).unwrap();
    let ret_data = [Felt::from(223)];
    cheatnet_state.start_mock_call(
        balance_contract_address,
        felt_selector_from_name("get_balance"),
        &ret_data,
    );

    let class_hash = declare(&mut cached_state, "ConstructorMockChecker", &contracts_data)
        .unwrap()
        .unwrap_success();
    let contract_address = deploy_wrapper(
        &mut cached_state,
        &mut cheatnet_state,
        &class_hash,
        &[balance_contract_address.into_()],
    )
    .unwrap();

    let selector = felt_selector_from_name("get_constructor_balance");

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );
    let output_data = recover_data(output);
    assert_eq!(output_data.len(), 1);
    assert_eq!(output_data.first().unwrap().clone(), Felt::from(223));
}

#[test]
fn mock_call_two_methods() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut cheatnet_state,
        "MockChecker",
        &[Felt::from(420)],
    );

    let selector1 = felt_selector_from_name("get_thing");
    let selector2 = felt_selector_from_name("get_constant_thing");

    let ret_data = [Felt::from(123)];
    cheatnet_state.start_mock_call(
        contract_address,
        felt_selector_from_name("get_thing"),
        &ret_data,
    );

    cheatnet_state.start_mock_call(
        contract_address,
        felt_selector_from_name("get_constant_thing"),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector1,
        &[],
    );

    assert_success(output, &ret_data);

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector2,
        &[],
    );

    assert_success(output, &ret_data);
}

#[test]
fn mock_call_nonexisting_contract() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let selector = felt_selector_from_name("get_thing");
    let ret_data = [Felt::from(123)];

    let contract_address = ContractAddress::from(218_u8);

    cheatnet_state.start_mock_call(
        contract_address,
        felt_selector_from_name("get_thing"),
        &ret_data,
    );

    let output = call_contract(
        &mut cached_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    assert_success(output, &ret_data);
}

#[test]
fn mock_call_simple_with_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("MockChecker", &[Felt::from(420)]);

    test_env.mock_call(
        &contract_address,
        "get_thing",
        &[123],
        CheatSpan::TargetCalls(2),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_thing", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_thing", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_thing", &[]),
        &[Felt::from(420)],
    );
}

#[test]
fn mock_call_proxy_with_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("MockChecker", &[Felt::from(420)]);
    let proxy_address = test_env.deploy("MockCheckerProxy", &[]);

    test_env.mock_call(
        &contract_address,
        "get_thing",
        &[123],
        CheatSpan::TargetCalls(2),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_thing", &[]),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(
            &proxy_address,
            "get_thing_from_contract",
            &[contract_address.into_()],
        ),
        &[Felt::from(123)],
    );
    assert_success(
        test_env.call_contract(
            &proxy_address,
            "get_thing_from_contract",
            &[contract_address.into_()],
        ),
        &[Felt::from(420)],
    );
}

#[test]
fn mock_call_in_constructor_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let balance_address = test_env.deploy("HelloStarknet", &[]);

    let class_hash = test_env.declare("ConstructorMockChecker", &contracts_data);
    let precalculated_address = test_env
        .cheatnet_state
        .precalculate_address(&class_hash, &[balance_address.into_()]);

    test_env.mock_call(
        &balance_address,
        "get_balance",
        &[111],
        CheatSpan::TargetCalls(2),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[balance_address.into_()]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_constructor_balance", &[]),
        &[Felt::from(111)],
    );
    assert_success(
        test_env.call_contract(&balance_address, "get_balance", &[]),
        &[Felt::from(111)],
    );
    assert_success(
        test_env.call_contract(&balance_address, "get_balance", &[]),
        &[Felt::from(0)],
    );
}

#[test]
fn mock_call_twice_in_function() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("MockChecker", &contracts_data);
    let precalculated_address = test_env
        .cheatnet_state
        .precalculate_address(&class_hash, &[111.into()]);

    test_env.mock_call(
        &precalculated_address,
        "get_thing",
        &[222],
        CheatSpan::TargetCalls(2),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[111.into()]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_thing", &[]),
        &[222.into()],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_thing_twice", &[]),
        &[222.into(), 111.into()],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_thing", &[]),
        &[111.into()],
    );
}

#[test]
fn mock_call_override_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("MockChecker", &[111.into()]);

    test_env.mock_call(
        &contract_address,
        "get_thing",
        &[222],
        CheatSpan::TargetCalls(2),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_thing", &[]),
        &[Felt::from(222)],
    );

    test_env.mock_call(
        &contract_address,
        "get_thing",
        &[333],
        CheatSpan::Indefinite,
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_thing", &[]),
        &[Felt::from(333)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_thing", &[]),
        &[Felt::from(333)],
    );

    test_env.stop_mock_call(&contract_address, "get_thing");

    assert_success(
        test_env.call_contract(&contract_address, "get_thing", &[]),
        &[111.into()],
    );
}
