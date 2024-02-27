use crate::common::assertions::assert_outputs;
use crate::common::state::build_runtime_state;
use crate::common::{call_contract, deploy_wrapper};
use crate::{
    assert_success,
    common::{
        deploy_contract, felt_selector_from_name, get_contracts, recover_data,
        state::create_cached_state,
    },
};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::state::{CheatTarget, CheatnetState};
use conversions::IntoConv;
use starknet_api::core::ContractAddress;

use super::test_environment::TestEnvironment;

const DEFAULT_BLOCK_TIMESTAMP: u64 = 0;

trait WarpTrait {
    fn start_warp(&mut self, target: CheatTarget, timestamp: u128);
    fn stop_warp(&mut self, contract_address: &ContractAddress);
}

impl<'a> WarpTrait for TestEnvironment<'a> {
    fn start_warp(&mut self, target: CheatTarget, timestamp: u128) {
        self.runtime_state
            .cheatnet_state
            .start_warp(target, Felt252::from(timestamp));
    }

    fn stop_warp(&mut self, contract_address: &ContractAddress) {
        self.runtime_state
            .cheatnet_state
            .stop_warp(CheatTarget::One(*contract_address));
    }
}

#[test]
fn warp_simple() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "WarpChecker", &[]);

    runtime_state
        .cheatnet_state
        .start_warp(CheatTarget::One(contract_address), Felt252::from(123_u128));

    let selector = felt_selector_from_name("get_block_timestamp");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn warp_with_other_syscall() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "WarpChecker", &[]);

    runtime_state
        .cheatnet_state
        .start_warp(CheatTarget::One(contract_address), Felt252::from(123));
    let selector = felt_selector_from_name("get_block_timestamp_and_emit_event");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn warp_in_constructor() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();

    let class_hash = declare(&mut cached_state, "ConstructorWarpChecker", &contracts).unwrap();
    let precalculated_address = runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);

    runtime_state
        .cheatnet_state
        .start_warp(CheatTarget::One(precalculated_address), Felt252::from(123));

    let contract_address =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    assert_eq!(precalculated_address, contract_address);

    let selector = felt_selector_from_name("get_stored_block_timestamp");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn warp_stop() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "WarpChecker", &[]);

    let selector = felt_selector_from_name("get_block_timestamp");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let old_block_timestamp = recover_data(output);

    runtime_state
        .cheatnet_state
        .start_warp(CheatTarget::One(contract_address), Felt252::from(123_u128));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let new_block_timestamp = recover_data(output);
    assert_eq!(new_block_timestamp, vec![Felt252::from(123)]);
    assert_ne!(old_block_timestamp, new_block_timestamp);

    runtime_state
        .cheatnet_state
        .stop_warp(CheatTarget::One(contract_address));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    let changed_back_block_timestamp = recover_data(output);

    assert_eq!(old_block_timestamp, changed_back_block_timestamp);
}

#[test]
fn warp_double() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "WarpChecker", &[]);

    let selector = felt_selector_from_name("get_block_timestamp");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let old_block_timestamp = recover_data(output);

    runtime_state
        .cheatnet_state
        .start_warp(CheatTarget::One(contract_address), Felt252::from(123_u128));
    runtime_state
        .cheatnet_state
        .start_warp(CheatTarget::One(contract_address), Felt252::from(123_u128));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let new_block_timestamp = recover_data(output);
    assert_eq!(new_block_timestamp, vec![Felt252::from(123)]);
    assert_ne!(old_block_timestamp, new_block_timestamp);

    runtime_state
        .cheatnet_state
        .stop_warp(CheatTarget::One(contract_address));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    let changed_back_block_timestamp = recover_data(output);

    assert_eq!(old_block_timestamp, changed_back_block_timestamp);
}

#[test]
fn warp_proxy() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "WarpChecker", &[]);

    let proxy_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "WarpCheckerProxy",
        &[],
    );
    let proxy_selector = felt_selector_from_name("get_warp_checkers_block_timestamp");
    let before_warp_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    );

    runtime_state
        .cheatnet_state
        .start_warp(CheatTarget::One(contract_address), Felt252::from(123_u128));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    );

    assert_success!(output, vec![Felt252::from(123)]);

    runtime_state
        .cheatnet_state
        .stop_warp(CheatTarget::One(contract_address));
    let after_warp_cancellation_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    );

    assert_outputs(before_warp_output, after_warp_cancellation_output);
}

#[test]
fn warp_library_call() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = declare(&mut cached_state, "WarpChecker", &contracts).unwrap();

    let lib_call_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "WarpCheckerLibCall",
        &[],
    );
    let lib_call_selector = felt_selector_from_name("get_block_timestamp_with_lib_call");

    let before_warp_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    );

    runtime_state
        .cheatnet_state
        .start_warp(CheatTarget::One(lib_call_address), Felt252::from(123_u128));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    );

    assert_success!(output, vec![Felt252::from(123)]);

    runtime_state
        .cheatnet_state
        .stop_warp(CheatTarget::One(lib_call_address));
    let after_warp_cancellation_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    );
    assert_outputs(before_warp_output, after_warp_cancellation_output);
}

#[test]
fn warp_all_simple() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = declare(&mut cached_state, "WarpChecker", &contracts).unwrap();

    let contract_address1 =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    let contract_address2 =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    runtime_state
        .cheatnet_state
        .start_warp(CheatTarget::All, Felt252::from(123));

    let selector = felt_selector_from_name("get_block_timestamp");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address1,
        &selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(123)]);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address2,
        &selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn warp_all_then_one() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "WarpChecker", &[]);

    runtime_state
        .cheatnet_state
        .start_warp(CheatTarget::All, Felt252::from(321_u128));
    runtime_state
        .cheatnet_state
        .start_warp(CheatTarget::One(contract_address), Felt252::from(123_u128));

    let selector = felt_selector_from_name("get_block_timestamp");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn warp_one_then_all() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "WarpChecker", &[]);

    runtime_state
        .cheatnet_state
        .start_warp(CheatTarget::One(contract_address), Felt252::from(123_u128));
    runtime_state
        .cheatnet_state
        .start_warp(CheatTarget::All, Felt252::from(321_u128));

    let selector = felt_selector_from_name("get_block_timestamp");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_success!(output, vec![Felt252::from(321)]);
}

#[test]
fn warp_all_stop() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "WarpChecker", &[]);

    let selector = felt_selector_from_name("get_block_timestamp");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let old_block_timestamp = recover_data(output);

    runtime_state
        .cheatnet_state
        .start_warp(CheatTarget::All, Felt252::from(123_u128));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let new_block_timestamp = recover_data(output);
    assert_eq!(new_block_timestamp, vec![Felt252::from(123)]);
    assert_ne!(old_block_timestamp, new_block_timestamp);

    runtime_state.cheatnet_state.stop_warp(CheatTarget::All);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    let changed_back_block_timestamp = recover_data(output);

    assert_eq!(old_block_timestamp, changed_back_block_timestamp);
}

#[test]
fn warp_multiple() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = declare(&mut cached_state, "WarpChecker", &contracts).unwrap();

    let contract_address1 =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    let contract_address2 =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    let selector = felt_selector_from_name("get_block_timestamp");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address1,
        &selector,
        &[],
    );

    let old_block_timestamp1 = recover_data(output);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address2,
        &selector,
        &[],
    );

    let old_block_timestamp2 = recover_data(output);

    runtime_state.cheatnet_state.start_warp(
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

    let new_block_timestamp1 = recover_data(output);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address2,
        &selector,
        &[],
    );

    let new_block_timestamp2 = recover_data(output);

    assert_eq!(new_block_timestamp1, vec![Felt252::from(123)]);
    assert_eq!(new_block_timestamp2, vec![Felt252::from(123)]);

    runtime_state
        .cheatnet_state
        .stop_warp(CheatTarget::Multiple(vec![
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

    let changed_back_block_timestamp1 = recover_data(output);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address2,
        &selector,
        &[],
    );

    let changed_back_block_timestamp2 = recover_data(output);

    assert_eq!(old_block_timestamp1, changed_back_block_timestamp1);
    assert_eq!(old_block_timestamp2, changed_back_block_timestamp2);
}

#[test]
fn warp_simple_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("WarpChecker", &[]);

    test_env.start_warp(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::Number(2),
    );

    assert_success!(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        vec![Felt252::from(123)]
    );
    assert_success!(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        vec![Felt252::from(123)]
    );
    assert_success!(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        vec![Felt252::from(DEFAULT_BLOCK_TIMESTAMP)]
    );
}

#[test]
fn warp_proxy_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = test_env.declare("WarpCheckerProxy", &contracts);
    let contract_address_1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address_2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.start_warp(
        CheatTarget::One(contract_address_1),
        123,
        CheatSpan::Number(1),
    );

    let output = test_env.call_contract(
        &contract_address_1,
        "call_proxy",
        &[contract_address_2.into_()],
    );
    assert_success!(output, vec![123.into(), DEFAULT_BLOCK_TIMESTAMP.into()]);
}

#[test]
fn warp_in_constructor_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();

    let class_hash = test_env.declare("ConstructorWarpChecker", &contracts);
    let precalculated_address = test_env
        .runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);

    test_env.start_warp(
        CheatTarget::One(precalculated_address),
        123,
        CheatSpan::Number(2),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success!(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        vec![Felt252::from(123)]
    );
    assert_success!(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        vec![Felt252::from(DEFAULT_BLOCK_TIMESTAMP)]
    );
    assert_success!(
        test_env.call_contract(&contract_address, "get_stored_block_timestamp", &[]),
        vec![Felt252::from(123)]
    );
}

#[test]
fn warp_no_contructor_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();

    let class_hash = test_env.declare("WarpChecker", &contracts);
    let precalculated_address = test_env
        .runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);

    test_env.start_warp(
        CheatTarget::One(precalculated_address),
        123,
        CheatSpan::Number(1),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success!(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        vec![Felt252::from(123)]
    );
    assert_success!(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        vec![Felt252::from(DEFAULT_BLOCK_TIMESTAMP)]
    );
}

#[test]
fn warp_override_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("WarpChecker", &[]);

    test_env.start_warp(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::Number(2),
    );

    assert_success!(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        vec![Felt252::from(123)]
    );

    test_env.start_warp(
        CheatTarget::One(contract_address),
        321,
        CheatSpan::Indefinite,
    );

    assert_success!(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        vec![Felt252::from(321)]
    );
    assert_success!(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        vec![Felt252::from(321)]
    );

    test_env.stop_warp(&contract_address);

    assert_success!(
        test_env.call_contract(&contract_address, "get_block_timestamp", &[]),
        vec![Felt252::from(DEFAULT_BLOCK_TIMESTAMP)]
    );
}

#[test]
fn warp_library_call_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = test_env.declare("WarpChecker", &contracts);
    let contract_address = test_env.deploy("WarpCheckerLibCall", &[]);

    test_env.start_warp(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::Number(1),
    );

    let lib_call_selector = "get_block_timestamp_with_lib_call";

    assert_success!(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        vec![Felt252::from(123)]
    );
    assert_success!(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        vec![Felt252::from(DEFAULT_BLOCK_TIMESTAMP)]
    );
}

#[test]
fn warp_all_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address_1 = test_env.deploy("WarpChecker", &[]);
    let contract_address_2 = test_env.deploy("WarpCheckerLibCall", &[]);

    test_env.start_warp(CheatTarget::All, 123, CheatSpan::Number(1));

    assert_success!(
        test_env.call_contract(&contract_address_1, "get_block_timestamp", &[]),
        vec![Felt252::from(123)]
    );
    assert_success!(
        test_env.call_contract(&contract_address_1, "get_block_timestamp", &[]),
        vec![Felt252::from(DEFAULT_BLOCK_TIMESTAMP)]
    );

    assert_success!(
        test_env.call_contract(&contract_address_2, "get_block_timestamp", &[]),
        vec![Felt252::from(123)]
    );
    assert_success!(
        test_env.call_contract(&contract_address_2, "get_block_timestamp", &[]),
        vec![Felt252::from(DEFAULT_BLOCK_TIMESTAMP)]
    );
}
