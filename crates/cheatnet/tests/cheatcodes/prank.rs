use crate::cheatcodes::spy_events::felt_vec_to_event_vec;
use crate::common::assertions::assert_success;
use crate::common::get_contracts;
use crate::common::state::build_runtime_state;
use blockifier::state::cached_state::{
    CachedState, GlobalContractCache, GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST,
};
use cairo_felt::{felt_str, Felt252};
use cairo_lang_starknet_classes::keccak::starknet_keccak;
use cheatnet::constants::build_testing_state;
use cheatnet::constants::TEST_ADDRESS;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::spy_events::{
    Event, SpyTarget,
};
use cheatnet::state::{CheatSpan, CheatTarget, CheatnetState, ExtendedStateReader};
use conversions::IntoConv;
use starknet_api::block::BlockNumber;
use starknet_api::core::{ContractAddress, PatriciaKey};
use starknet_api::hash::StarkHash;
use starknet_api::{contract_address, patricia_key};
use tempfile::TempDir;

use super::test_environment::TestEnvironment;

trait PrankTrait {
    fn prank(&mut self, target: CheatTarget, new_address: u128, span: CheatSpan);
    fn start_prank(&mut self, target: CheatTarget, new_address: u128);
    fn stop_prank(&mut self, contract_address: &ContractAddress);
}

impl<'a> PrankTrait for TestEnvironment<'a> {
    fn prank(&mut self, target: CheatTarget, new_address: u128, span: CheatSpan) {
        self.runtime_state
            .cheatnet_state
            .prank(target, ContractAddress::from(new_address), span);
    }

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
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("PrankChecker", &[]);

    test_env.start_prank(CheatTarget::One(contract_address), 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn prank_with_other_syscall() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("PrankChecker", &[]);

    test_env.start_prank(CheatTarget::One(contract_address), 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address_and_emit_event", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn prank_in_constructor() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("ConstructorPrankChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.start_prank(CheatTarget::One(precalculated_address), 123);

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_stored_caller_address", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn prank_stop() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("PrankChecker", &[]);

    test_env.start_prank(CheatTarget::One(contract_address), 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt252::from(123)],
    );

    test_env.stop_prank(&contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[contract_address!(TEST_ADDRESS).into_()],
    );
}

#[test]
fn prank_double() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("PrankChecker", &[]);

    test_env.start_prank(CheatTarget::One(contract_address), 111);
    test_env.start_prank(CheatTarget::One(contract_address), 222);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt252::from(222)],
    );

    test_env.stop_prank(&contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[contract_address!(TEST_ADDRESS).into_()],
    );
}

#[test]
fn prank_proxy() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("PrankChecker", &[]);
    let proxy_address = test_env.deploy("PrankCheckerProxy", &[]);

    test_env.start_prank(CheatTarget::One(contract_address), 123);

    let selector = "get_prank_checkers_caller_address";
    assert_success(
        test_env.call_contract(&proxy_address, selector, &[contract_address.into_()]),
        &[Felt252::from(123)],
    );

    test_env.stop_prank(&contract_address);

    assert_success(
        test_env.call_contract(&proxy_address, selector, &[contract_address.into_()]),
        &[proxy_address.into_()],
    );
}

#[test]
fn prank_library_call() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("PrankChecker", &contracts_data);
    let lib_call_address = test_env.deploy("PrankCheckerLibCall", &[]);

    test_env.start_prank(CheatTarget::One(lib_call_address), 123);

    let lib_call_selector = "get_caller_address_with_lib_call";
    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(123)],
    );

    test_env.stop_prank(&lib_call_address);

    assert_success(
        test_env.call_contract(&lib_call_address, lib_call_selector, &[class_hash.into_()]),
        &[contract_address!(TEST_ADDRESS).into_()],
    );
}

#[test]
fn prank_all() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("PrankChecker", &[]);

    test_env.start_prank(CheatTarget::All, 123);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt252::from(123)],
    );

    test_env
        .runtime_state
        .cheatnet_state
        .stop_prank(CheatTarget::All);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[contract_address!(TEST_ADDRESS).into_()],
    );
}

#[test]
fn prank_multiple() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("PrankChecker", &contracts_data);

    let contract_address1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.start_prank(
        CheatTarget::Multiple(vec![contract_address1, contract_address2]),
        123,
    );

    assert_success(
        test_env.call_contract(&contract_address1, "get_caller_address", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_caller_address", &[]),
        &[Felt252::from(123)],
    );

    test_env
        .runtime_state
        .cheatnet_state
        .stop_prank(CheatTarget::Multiple(vec![
            contract_address1,
            contract_address2,
        ]));

    assert_success(
        test_env.call_contract(&contract_address1, "get_caller_address", &[]),
        &[contract_address!(TEST_ADDRESS).into_()],
    );
    assert_success(
        test_env.call_contract(&contract_address2, "get_caller_address", &[]),
        &[contract_address!(TEST_ADDRESS).into_()],
    );
}

#[test]
fn prank_all_then_one() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("PrankChecker", &[]);

    test_env.start_prank(CheatTarget::All, 111);
    test_env.start_prank(CheatTarget::One(contract_address), 222);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt252::from(222)],
    );
}

#[test]
fn prank_one_then_all() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("PrankChecker", &[]);

    test_env.start_prank(CheatTarget::One(contract_address), 111);
    test_env.start_prank(CheatTarget::All, 222);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt252::from(222)],
    );
}

#[ignore] // TODO (#1916)
#[test]
fn prank_cairo0_callback() {
    let temp_dir = TempDir::new().unwrap();
    let cached_state = CachedState::new(
        ExtendedStateReader {
            dict_state_reader: build_testing_state(),
            fork_state_reader: Some(ForkStateReader::new(
                "http://188.34.188.184:6060/rpc/v0_7".parse().unwrap(),
                BlockNumber(950_486),
                temp_dir.path().to_str().unwrap(),
            )),
        },
        GlobalContractCache::new(GLOBAL_CONTRACT_CACHE_SIZE_FOR_TEST),
    );
    let mut cheatnet_state = CheatnetState::default();
    let runtime_state = build_runtime_state(&mut cheatnet_state);
    let mut test_env = TestEnvironment {
        cached_state,
        runtime_state,
    };

    let contract_address = test_env.deploy("Cairo1Contract_v1", &[]);

    test_env.start_prank(CheatTarget::One(contract_address), 123);
    let id = test_env
        .runtime_state
        .cheatnet_state
        .spy_events(SpyTarget::All);

    let expected_caller_address = Felt252::from(123);

    assert_success(
        test_env.call_contract(
            &contract_address,
            "start",
            &[
                felt_str!(
                    // cairo 0 callback contract address
                    "034dad9a1512fcb0d33032c65f4605a073bdc42f70e61524510e5760c2b4f544",
                    16
                ),
                expected_caller_address.clone(),
            ],
        ),
        &[],
    );

    let (_, events) = test_env
        .runtime_state
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
}

#[test]
fn prank_simple_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("PrankChecker", &[]);

    test_env.prank(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::TargetCalls(2),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[contract_address!(TEST_ADDRESS).into_()],
    );
}

#[test]
fn prank_proxy_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("PrankCheckerProxy", &contracts_data);

    let contract_address_1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address_2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.prank(
        CheatTarget::One(contract_address_1),
        123,
        CheatSpan::TargetCalls(1),
    );

    let output = test_env.call_contract(
        &contract_address_1,
        "call_proxy",
        &[contract_address_2.into_()],
    );
    assert_success(output, &[123.into(), contract_address_2.into_()]);
}

#[test]
fn prank_override_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address = test_env.deploy("PrankChecker", &[]);

    test_env.prank(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::TargetCalls(2),
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt252::from(123)],
    );

    test_env.prank(
        CheatTarget::One(contract_address),
        321,
        CheatSpan::Indefinite,
    );

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt252::from(321)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt252::from(321)],
    );

    test_env.stop_prank(&contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[contract_address!(TEST_ADDRESS).into_()],
    );
}

#[test]
fn prank_constructor_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("ConstructorPrankChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.prank(
        CheatTarget::One(precalculated_address),
        123,
        CheatSpan::TargetCalls(3),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_stored_caller_address", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_caller_address", &[]),
        &[contract_address!(TEST_ADDRESS).into_()],
    );
}

#[test]
fn prank_library_call_with_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("PrankChecker", &contracts_data);
    let contract_address = test_env.deploy("PrankCheckerLibCall", &[]);

    test_env.prank(
        CheatTarget::One(contract_address),
        123,
        CheatSpan::TargetCalls(1),
    );

    let lib_call_selector = "get_caller_address_with_lib_call";

    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[contract_address!(TEST_ADDRESS).into_()],
    );
}

#[test]
fn prank_all_span() {
    let mut cheatnet_state = CheatnetState::default();
    let mut test_env = TestEnvironment::new(&mut cheatnet_state);

    let contract_address_1 = test_env.deploy("PrankChecker", &[]);
    let contract_address_2 = test_env.deploy("PrankCheckerLibCall", &[]);

    test_env.prank(CheatTarget::All, 123, CheatSpan::TargetCalls(1));

    assert_success(
        test_env.call_contract(&contract_address_1, "get_caller_address", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address_1, "get_caller_address", &[]),
        &[contract_address!(TEST_ADDRESS).into_()],
    );

    assert_success(
        test_env.call_contract(&contract_address_2, "get_caller_address", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address_2, "get_caller_address", &[]),
        &[contract_address!(TEST_ADDRESS).into_()],
    );
}
