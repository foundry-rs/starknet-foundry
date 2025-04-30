use crate::common::call_contract;
use crate::common::state::create_fork_cached_state;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::rpc::CallResult;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::selector_from_name;
use cheatnet::state::CheatnetState;
use conversions::string::TryFromHexStr;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;
use tempfile::TempDir;
use test_case::test_case;

const CAIRO0_TESTER_ADDRESS: &str =
    "0x7fec0c04dde6b1cfa7994359313f8b67edd0d8e40e28424437702d3ee48c2a4";

#[test_case("return_caller_address"; "when common call")]
#[test_case("return_proxied_caller_address"; "when library call")]
fn cheat_caller_address_cairo0_contract(selector: &str) {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = ContractAddress::try_from_hex_str(CAIRO0_TESTER_ADDRESS).unwrap();

    let selector = selector_from_name(selector);
    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );

    let CallResult::Success { ret_data } = output else {
        panic!("Wrong call output")
    };
    let caller = &ret_data[0];

    cheatnet_state.start_cheat_caller_address(contract_address, ContractAddress::from(123_u128));

    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );
    let CallResult::Success { ret_data } = output else {
        panic!("Wrong call output")
    };
    let cheated_caller_address = &ret_data[0];

    cheatnet_state.stop_cheat_caller_address(contract_address);

    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );
    let CallResult::Success { ret_data } = output else {
        panic!("Wrong call output")
    };
    let uncheated_caller_address = &ret_data[0];

    assert_eq!(cheated_caller_address, &Felt::from(123));
    assert_eq!(uncheated_caller_address, caller);
}

#[test_case("return_block_number"; "when common call")]
#[test_case("return_proxied_block_number"; "when library call")]
fn cheat_block_number_cairo0_contract(selector: &str) {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = ContractAddress::try_from_hex_str(CAIRO0_TESTER_ADDRESS).unwrap();

    let selector = selector_from_name(selector);
    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );
    let CallResult::Success { ret_data } = output else {
        panic!("Wrong call output")
    };
    let block_number = &ret_data[0];

    cheatnet_state.start_cheat_block_number(contract_address, 123);

    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );
    let CallResult::Success { ret_data } = output else {
        panic!("Wrong call output")
    };
    let cheated_block_number = &ret_data[0];

    cheatnet_state.stop_cheat_block_number(contract_address);

    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );
    let CallResult::Success { ret_data } = output else {
        panic!("Wrong call output")
    };
    let uncheated_block_number = &ret_data[0];

    assert_eq!(cheated_block_number, &Felt::from(123));
    assert_eq!(uncheated_block_number, block_number);
}

#[test_case("return_block_timestamp"; "when common call")]
#[test_case("return_proxied_block_timestamp"; "when library call")]
fn cheat_block_timestamp_cairo0_contract(selector: &str) {
    let cache_dir = TempDir::new().unwrap();
    let mut cached_fork_state = create_fork_cached_state(cache_dir.path().to_str().unwrap());
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = ContractAddress::try_from_hex_str(CAIRO0_TESTER_ADDRESS).unwrap();

    let selector = selector_from_name(selector);
    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );
    let CallResult::Success { ret_data } = output else {
        panic!("Wrong call output")
    };
    let block_timestamp = &ret_data[0];

    cheatnet_state.start_cheat_block_timestamp(contract_address, 123);
    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );
    let CallResult::Success { ret_data } = output else {
        panic!("Wrong call output")
    };
    let cheated_block_timestamp = &ret_data[0];

    cheatnet_state.stop_cheat_block_timestamp(contract_address);

    let output = call_contract(
        &mut cached_fork_state,
        &mut cheatnet_state,
        &contract_address,
        selector,
        &[],
    );
    let CallResult::Success { ret_data } = output else {
        panic!("Wrong call output")
    };
    let uncheated_block_timestamp = &ret_data[0];

    assert_eq!(cheated_block_timestamp, &Felt::from(123));
    assert_eq!(uncheated_block_timestamp, block_timestamp);
}
