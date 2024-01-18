use crate::common::state::{create_cheatnet_state, create_fork_cached_state};
use crate::common::{call_contract, felt_selector_from_name};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallResult;
use cheatnet::state::CheatTarget;
use conversions::IntoConv;
use num_bigint::BigUint;
use starknet_api::core::ContractAddress;
use std::str::FromStr;
use tempfile::TempDir;
use test_case::test_case;

const CAIRO0_TESTER_ADDRESS: &str =
    "1825832089891106126806210124294467331434544162488231781791271899226056323189";

#[test_case("return_caller_address"; "when common call")]
#[test_case("return_proxied_caller_address"; "when library call")]
fn prank_cairo0_contract(selector: &str) {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_fork_state);

    let contract_address = Felt252::from(BigUint::from_str(CAIRO0_TESTER_ADDRESS).unwrap()).into_();

    let selector = felt_selector_from_name(selector);
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let CallResult::Success { ret_data } = output.result else {
        panic!("Wrong call output")
    };
    let caller = &ret_data[0];

    cheatnet_state.start_prank(
        CheatTarget::One(contract_address),
        ContractAddress::from(123_u128),
    );

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let CallResult::Success { ret_data } = output.result else {
        panic!("Wrong call output")
    };
    let pranked_caller = &ret_data[0];

    cheatnet_state.stop_prank(CheatTarget::One(contract_address));

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let CallResult::Success { ret_data } = output.result else {
        panic!("Wrong call output")
    };
    let unpranked_caller = &ret_data[0];

    assert_eq!(pranked_caller, &Felt252::from(123));
    assert_eq!(unpranked_caller, caller);
}

#[test_case("return_block_number"; "when common call")]
#[test_case("return_proxied_block_number"; "when library call")]
fn roll_cairo0_contract(selector: &str) {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_fork_state);

    let contract_address = Felt252::from(BigUint::from_str(CAIRO0_TESTER_ADDRESS).unwrap()).into_();

    let selector = felt_selector_from_name(selector);
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let CallResult::Success { ret_data } = output.result else {
        panic!("Wrong call output")
    };
    let block_number = &ret_data[0];

    cheatnet_state.start_roll(CheatTarget::One(contract_address), Felt252::from(123));

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let CallResult::Success { ret_data } = output.result else {
        panic!("Wrong call output")
    };
    let rolled_block_number = &ret_data[0];

    cheatnet_state.stop_roll(CheatTarget::One(contract_address));

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let CallResult::Success { ret_data } = output.result else {
        panic!("Wrong call output")
    };
    let unrolled_block_number = &ret_data[0];

    assert_eq!(rolled_block_number, &Felt252::from(123));
    assert_eq!(unrolled_block_number, block_number);
}

#[test_case("return_block_timestamp"; "when common call")]
#[test_case("return_proxied_block_timestamp"; "when library call")]
fn warp_cairo0_contract(selector: &str) {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_fork_state);

    let contract_address = Felt252::from(BigUint::from_str(CAIRO0_TESTER_ADDRESS).unwrap()).into_();

    let selector = felt_selector_from_name(selector);
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let CallResult::Success { ret_data } = output.result else {
        panic!("Wrong call output")
    };
    let block_timestamp = &ret_data[0];

    cheatnet_state.start_warp(CheatTarget::One(contract_address), Felt252::from(123));
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let CallResult::Success { ret_data } = output.result else {
        panic!("Wrong call output")
    };
    let warped_block_timestamp = &ret_data[0];

    cheatnet_state.stop_warp(CheatTarget::One(contract_address));

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[],
    )
    .unwrap();
    let CallResult::Success { ret_data } = output.result else {
        panic!("Wrong call output")
    };
    let unwarped_block_timestamp = &ret_data[0];

    assert_eq!(warped_block_timestamp, &Felt252::from(123));
    assert_eq!(unwarped_block_timestamp, block_timestamp);
}
