use crate::common::assertions::assert_outputs;
use crate::common::state::build_runtime_state;
use crate::common::{call_contract, deploy_wrapper};
use crate::{
    common::assertions::assert_success,
    common::{
        deploy_contract, felt_selector_from_name, get_contracts, recover_data,
        state::create_cached_state,
    },
};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::state::{CheatSpan, CheatTarget, CheatnetState};
use conversions::IntoConv;
use runtime::starknet::context::DEFAULT_BLOCK_NUMBER;
use starknet_api::core::ContractAddress;

use super::test_environment::TestEnvironment;

trait RollTrait {
    fn roll(&mut self, target: CheatTarget, block_number: u128, span: CheatSpan);
    fn start_roll(&mut self, target: CheatTarget, block_number: u128);
    fn stop_roll(&mut self, contract_address: &ContractAddress);
}

impl<'a> RollTrait for TestEnvironment<'a> {
    fn roll(&mut self, target: CheatTarget, block_number: u128, span: CheatSpan) {
        self.runtime_state
            .cheatnet_state
            .roll(target, Felt252::from(block_number), span);
    }

    fn start_roll(&mut self, target: CheatTarget, block_number: u128) {
        self.runtime_state
            .cheatnet_state
            .start_roll(target, Felt252::from(block_number));
    }

    fn stop_roll(&mut self, contract_address: &ContractAddress) {
        self.runtime_state
            .cheatnet_state
            .stop_roll(CheatTarget::One(*contract_address));
    }
}

#[test]
fn roll_simple() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "RollChecker", &[]);

    runtime_state
        .cheatnet_state
        .start_roll(CheatTarget::One(contract_address), Felt252::from(123_u128));

    let selector = felt_selector_from_name("get_block_number");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn roll_with_other_syscall() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "RollChecker", &[]);

    runtime_state
        .cheatnet_state
        .start_roll(CheatTarget::One(contract_address), Felt252::from(123_u128));

    let selector = felt_selector_from_name("get_block_number_and_emit_event");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn roll_in_constructor() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();

    let class_hash = declare(&mut cached_state, "ConstructorRollChecker", &contracts).unwrap();
    let precalculated_address = runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);

    runtime_state.cheatnet_state.start_roll(
        CheatTarget::One(precalculated_address),
        Felt252::from(123_u128),
    );

    let contract_address =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    assert_eq!(precalculated_address, contract_address);

    let selector = felt_selector_from_name("get_stored_block_number");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn roll_stop() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "RollChecker", &[]);

    let selector = felt_selector_from_name("get_block_number");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let old_block_number = recover_data(output);

    runtime_state
        .cheatnet_state
        .start_roll(CheatTarget::One(contract_address), Felt252::from(123_u128));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let new_block_number = recover_data(output);
    assert_eq!(new_block_number, &[Felt252::from(123)]);
    assert_ne!(old_block_number, new_block_number);

    runtime_state
        .cheatnet_state
        .stop_roll(CheatTarget::One(contract_address));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    let changed_back_block_number = recover_data(output);

    assert_eq!(old_block_number, changed_back_block_number);
}

#[test]
fn roll_double() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "RollChecker", &[]);

    let selector = felt_selector_from_name("get_block_number");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let old_block_number = recover_data(output);

    runtime_state
        .cheatnet_state
        .start_roll(CheatTarget::One(contract_address), Felt252::from(123_u128));
    runtime_state
        .cheatnet_state
        .start_roll(CheatTarget::One(contract_address), Felt252::from(123_u128));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let new_block_number = recover_data(output);
    assert_eq!(new_block_number, &[Felt252::from(123)]);
    assert_ne!(old_block_number, new_block_number);

    runtime_state
        .cheatnet_state
        .stop_roll(CheatTarget::One(contract_address));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    let changed_back_block_number = recover_data(output);

    assert_eq!(old_block_number, changed_back_block_number);
}

#[test]
fn roll_proxy() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "RollChecker", &[]);

    let proxy_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "RollCheckerProxy",
        &[],
    );

    let proxy_selector = felt_selector_from_name("get_roll_checkers_block_number");
    let before_roll_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    );

    runtime_state
        .cheatnet_state
        .start_roll(CheatTarget::One(contract_address), Felt252::from(123_u128));

    let after_roll_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    );

    assert_success(after_roll_output, &[Felt252::from(123)]);

    runtime_state
        .cheatnet_state
        .stop_roll(CheatTarget::One(contract_address));

    let after_roll_cancellation_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    );

    assert_outputs(before_roll_output, after_roll_cancellation_output);
}

#[test]
fn roll_library_call() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = declare(&mut cached_state, "RollChecker", &contracts).unwrap();

    let lib_call_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "RollCheckerLibCall",
        &[],
    );

    let lib_call_selector = felt_selector_from_name("get_block_number_with_lib_call");
    let before_roll_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    );

    runtime_state
        .cheatnet_state
        .start_roll(CheatTarget::One(lib_call_address), Felt252::from(123_u128));

    let after_roll_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    );

    assert_success(after_roll_output, &[Felt252::from(123)]);

    runtime_state
        .cheatnet_state
        .stop_roll(CheatTarget::One(lib_call_address));

    let after_roll_cancellation_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    );

    assert_outputs(before_roll_output, after_roll_cancellation_output);
}

#[test]
fn roll_all_simple() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "RollChecker", &[]);

    runtime_state
        .cheatnet_state
        .start_roll(CheatTarget::All, Felt252::from(123));

    let selector = felt_selector_from_name("get_block_number");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn roll_all_then_one() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "RollChecker", &[]);

    runtime_state
        .cheatnet_state
        .start_roll(CheatTarget::All, Felt252::from(321_u128));
    runtime_state
        .cheatnet_state
        .start_roll(CheatTarget::One(contract_address), Felt252::from(123_u128));

    let selector = felt_selector_from_name("get_block_number");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn roll_one_then_all() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "RollChecker", &[]);

    runtime_state
        .cheatnet_state
        .start_roll(CheatTarget::One(contract_address), Felt252::from(123_u128));
    runtime_state
        .cheatnet_state
        .start_roll(CheatTarget::All, Felt252::from(321_u128));

    let selector = felt_selector_from_name("get_block_number");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success(output, &[Felt252::from(321)]);
}

#[test]
fn roll_all_stop() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "RollChecker", &[]);

    let selector = felt_selector_from_name("get_block_number");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let old_block_number = recover_data(output);

    runtime_state
        .cheatnet_state
        .start_roll(CheatTarget::All, Felt252::from(123_u128));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let new_block_number = recover_data(output);
    assert_eq!(new_block_number, &[Felt252::from(123)]);
    assert_ne!(old_block_number, new_block_number);

    runtime_state.cheatnet_state.stop_roll(CheatTarget::All);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    let changed_back_block_number = recover_data(output);

    assert_eq!(old_block_number, changed_back_block_number);
}

#[test]
fn roll_multiple() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = declare(&mut cached_state, "RollChecker", &contracts).unwrap();

    let contract_address1 =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    let contract_address2 =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    let selector = felt_selector_from_name("get_block_number");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address1,
        &selector,
        &[],
    );

    let old_block_number1 = recover_data(output);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address2,
        &selector,
        &[],
    );

    let old_block_number2 = recover_data(output);

    runtime_state.cheatnet_state.start_roll(
        CheatTarget::Multiple(vec![contract_address1, contract_address2]),
        Felt252::from(123_u128),
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address1,
        &selector,
        &[],
    );

    let new_block_number1 = recover_data(output);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address2,
        &selector,
        &[],
    );

    let new_block_number2 = recover_data(output);

    assert_eq!(new_block_number1, &[Felt252::from(123)]);
    assert_eq!(new_block_number2, &[Felt252::from(123)]);

    runtime_state
        .cheatnet_state
        .stop_roll(CheatTarget::Multiple(vec![
            contract_address1,
            contract_address2,
        ]));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address1,
        &selector,
        &[],
    );

    let changed_back_block_number1 = recover_data(output);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address2,
        &selector,
        &[],
    );

    let changed_back_block_number2 = recover_data(output);

    assert_eq!(old_block_number1, changed_back_block_number1);
    assert_eq!(old_block_number2, changed_back_block_number2);
}

#[test]
fn roll_simple_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.roll(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::Number(2),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_proxy_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = test_env.declare("RollCheckerProxy", &contracts);
    let contract_address_1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address_2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.roll(
        CheatTarget::One(contract_address_1),
        123,
        CheatSpan::Number(1),
    );

    let output = test_env.call_contract(
        &contract_address_1,
        "call_proxy",
        &[contract_address_2.into_()],
    );
    assert_success(output, &[123.into(), DEFAULT_BLOCK_NUMBER.into()]);
}

#[test]
fn roll_in_constructor_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();

    let class_hash = test_env.declare("ConstructorRollChecker", &contracts);
    let precalculated_address = test_env
        .runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);

    test_env.roll(
        CheatTarget::One(precalculated_address),
        123,
        CheatSpan::Number(2),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_stored_block_number", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn roll_no_constructor_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();

    let class_hash = test_env.declare("RollChecker", &contracts);
    let precalculated_address = test_env
        .runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);

    test_env.roll(
        CheatTarget::One(precalculated_address),
        123,
        CheatSpan::Number(1),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_override_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("RollChecker", &[]);

    test_env.roll(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::Number(2),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(123)],
    );

    test_env.roll(
        CheatTarget::One(contract_address),
        321,
        CheatSpan::Indefinite,
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(321)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(321)],
    );

    test_env.stop_roll(&contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_library_call_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = test_env.declare("RollChecker", &contracts);
    let contract_address = test_env.deploy("RollCheckerLibCall", &[]);

    test_env.roll(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::Number(1),
    );

    let lib_call_selector = "get_block_number_with_lib_call";

    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}

#[test]
fn roll_all_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address_1 = test_env.deploy("RollChecker", &[]);
    let contract_address_2 = test_env.deploy("RollCheckerLibCall", &[]);

    test_env.roll(CheatTarget::All, 123, CheatSpan::Number(1));

    assert_success(
        test_env.call_contract(&contract_address_1, "get_block_number", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address_1, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );

    assert_success(
        test_env.call_contract(&contract_address_2, "get_block_number", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address_2, "get_block_number", &[]),
        &[Felt252::from(DEFAULT_BLOCK_NUMBER)],
    );
}
