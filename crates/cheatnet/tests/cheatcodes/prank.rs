use crate::cheatcodes::spy_events::felt_vec_to_event_vec;
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
use blockifier::state::cached_state::{CachedState, GlobalContractCache};
use cairo_felt::{felt_str, Felt252};
use cairo_lang_starknet::contract::starknet_keccak;
use cheatnet::constants::build_testing_state;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::{
    Event, SpyTarget,
};
use cheatnet::state::{CheatTarget, CheatnetState, ExtendedStateReader};
use conversions::IntoConv;
use starknet_api::block::BlockNumber;
use starknet_api::core::ContractAddress;
use tempfile::TempDir;

use super::test_environment::TestEnvironment;

trait PrankTrait {
    fn start_prank(&mut self, target: CheatTarget, new_address: u128);
    fn stop_prank(&mut self, contract_address: &ContractAddress);
}

impl<'a> PrankTrait for TestEnvironment<'a> {
    fn start_prank(&mut self, target: CheatTarget, new_address: u128) {
        self.runtime_state
            .cheatnet_state
            .start_prank(target, ContractAddress::from(new_address));
    }

    fn stop_prank(&mut self, contract_address: &ContractAddress) {
        self.runtime_state
            .cheatnet_state
            .stop_prank(CheatTarget::One(*contract_address));
    }
}

#[test]
fn prank_simple() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "PrankChecker", &[]);

    runtime_state.cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let selector = felt_selector_from_name("get_caller_address");

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
fn prank_with_other_syscall() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "PrankChecker", &[]);

    runtime_state.cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let selector = felt_selector_from_name("get_caller_address_and_emit_event");

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
fn prank_in_constructor() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();

    let class_hash = declare(&mut cached_state, "ConstructorPrankChecker", &contracts).unwrap();
    let precalculated_address = runtime_state
        .cheatnet_state
        .precalculate_address(&class_hash, &[]);

    runtime_state.cheatnet_state.start_prank(
        CheatTarget::One(precalculated_address),
        ContractAddress::from(123_u128),
    );

    let contract_address =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    assert_eq!(precalculated_address, contract_address);

    let selector = felt_selector_from_name("get_stored_caller_address");

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
fn prank_stop() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "PrankChecker", &[]);

    let selector = felt_selector_from_name("get_caller_address");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let old_address = recover_data(output);

    runtime_state.cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let new_address = recover_data(output);
    assert_eq!(new_address, vec![Felt252::from(123)]);
    assert_ne!(old_address, new_address);

    runtime_state
        .cheatnet_state
        .stop_prank(CheatTarget::One(contract_address));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    let changed_back_address = recover_data(output);

    assert_eq!(old_address, changed_back_address);
}

#[test]
fn prank_double() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "PrankChecker", &[]);

    let selector = felt_selector_from_name("get_caller_address");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let old_address = recover_data(output);

    runtime_state.cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );
    runtime_state.cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let new_address = recover_data(output);
    assert_eq!(new_address, vec![Felt252::from(123)]);
    assert_ne!(old_address, new_address);

    runtime_state
        .cheatnet_state
        .stop_prank(CheatTarget::One(contract_address));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    let changed_back_address = recover_data(output);

    assert_eq!(old_address, changed_back_address);
}

#[test]
fn prank_proxy() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "PrankChecker", &[]);

    let proxy_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "PrankCheckerProxy",
        &[],
    );

    let proxy_selector = felt_selector_from_name("get_prank_checkers_caller_address");
    let before_prank_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    );

    runtime_state.cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let after_prank_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    );

    assert_success!(after_prank_output, vec![Felt252::from(123)]);

    runtime_state
        .cheatnet_state
        .stop_prank(CheatTarget::One(contract_address));

    let after_prank_cancellation_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    );

    assert_outputs(before_prank_output, after_prank_cancellation_output);
}

#[test]
fn prank_library_call() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = declare(&mut cached_state, "PrankChecker", &contracts).unwrap();

    let lib_call_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "PrankCheckerLibCall",
        &[],
    );

    let lib_call_selector = felt_selector_from_name("get_caller_address_with_lib_call");
    let before_prank_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    );

    runtime_state.cheatnet_state.start_prank(
        CheatTarget::One(lib_call_address),
        ContractAddress::from(123_u128),
    );

    let after_prank_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    );

    assert_success!(after_prank_output, vec![Felt252::from(123)]);

    runtime_state
        .cheatnet_state
        .stop_prank(CheatTarget::One(lib_call_address));

    let after_prank_cancellation_output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    );

    assert_outputs(before_prank_output, after_prank_cancellation_output);
}

#[test]
fn prank_all() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "PrankChecker", &[]);

    let selector = felt_selector_from_name("get_caller_address");

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let old_address = recover_data(output);

    runtime_state
        .cheatnet_state
        .start_prank(CheatTarget::All, ContractAddress::from(123_u128));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    let new_address = recover_data(output);
    assert_eq!(new_address, vec![Felt252::from(123)]);
    assert_ne!(old_address, new_address);

    runtime_state.cheatnet_state.stop_prank(CheatTarget::All);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );
    let changed_back_address = recover_data(output);

    assert_eq!(old_address, changed_back_address);
}

#[test]
fn prank_multiple() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contracts = get_contracts();
    let class_hash = declare(&mut cached_state, "PrankChecker", &contracts).unwrap();

    let contract_address1 =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    let contract_address2 =
        deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address1,
        &felt_selector_from_name("get_caller_address"),
        &[],
    );

    let old_address1 = recover_data(output);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address2,
        &felt_selector_from_name("get_caller_address"),
        &[],
    );

    let old_address2 = recover_data(output);

    runtime_state.cheatnet_state.start_prank(
        CheatTarget::Multiple(vec![contract_address1, contract_address2]),
        ContractAddress::from(123_u128),
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address1,
        &felt_selector_from_name("get_caller_address"),
        &[],
    );

    let new_address1 = recover_data(output);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address2,
        &felt_selector_from_name("get_caller_address"),
        &[],
    );

    let new_address2 = recover_data(output);

    assert_eq!(new_address1, vec![Felt252::from(123)]);
    assert_eq!(new_address2, vec![Felt252::from(123)]);

    runtime_state
        .cheatnet_state
        .stop_prank(CheatTarget::Multiple(vec![
            contract_address1,
            contract_address2,
        ]));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address1,
        &felt_selector_from_name("get_caller_address"),
        &[],
    );

    let changed_back_address1 = recover_data(output);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address2,
        &felt_selector_from_name("get_caller_address"),
        &[],
    );

    let changed_back_address2 = recover_data(output);

    assert_eq!(old_address1, changed_back_address1);
    assert_eq!(old_address2, changed_back_address2);
}

#[test]
fn prank_all_then_one() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "PrankChecker", &[]);

    let selector = felt_selector_from_name("get_caller_address");

    runtime_state
        .cheatnet_state
        .start_prank(CheatTarget::All, ContractAddress::from(321_u128));
    runtime_state.cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_eq!(recover_data(output), vec![Felt252::from(123)]);
}

#[test]
fn prank_one_then_all() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "PrankChecker", &[]);

    let selector = felt_selector_from_name("get_caller_address");

    runtime_state.cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );
    runtime_state
        .cheatnet_state
        .start_prank(CheatTarget::All, ContractAddress::from(321_u128));

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &selector,
        &[],
    );

    assert_eq!(recover_data(output), vec![Felt252::from(321)]);
}

#[test]
fn prank_cairo0_callback() {
    let temp_dir = TempDir::new().unwrap();
    let mut cached_state = CachedState::new(
        ExtendedStateReader {
            dict_state_reader: build_testing_state(),
            fork_state_reader: Some(ForkStateReader::new(
                "http://188.34.188.184:6060/rpc/v0_6".parse().unwrap(),
                BlockNumber(950_486),
                temp_dir.path().to_str().unwrap(),
            )),
        },
        GlobalContractCache::default(),
    );
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address = deploy_contract(
        &mut cached_state,
        &mut runtime_state,
        "Cairo1Contract_v1",
        &[],
    );

    runtime_state.cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );
    let id = runtime_state.cheatnet_state.spy_events(SpyTarget::All);

    let expected_caller_address = Felt252::from(123_u128);

    let output = call_contract(
        &mut cached_state,
        &mut runtime_state,
        &contract_address,
        &felt_selector_from_name("start"),
        &[
            felt_str!(
                // cairo 0 callback contract address
                "034dad9a1512fcb0d33032c65f4605a073bdc42f70e61524510e5760c2b4f544",
                16
            ),
            expected_caller_address.clone(),
        ],
    );

    let (_, events) = runtime_state
        .cheatnet_state
        .fetch_events(&Felt252::from(id));

    let events = felt_vec_to_event_vec(&events);

    // make sure end() was called by cairo0 contract
    assert_eq!(
        events[0],
        Event {
            from: contract_address,
            keys: vec![starknet_keccak("End".as_ref()).into()],
            data: vec![expected_caller_address]
        },
        "Wrong event"
    );

    assert_success!(output, vec![]);
}
