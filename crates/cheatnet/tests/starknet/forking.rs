use crate::common::state::create_cheatnet_fork_state;
use crate::common::{deploy_contract, felt_selector_from_name};
use crate::{assert_error, assert_success};
use blockifier::state::cached_state::CachedState;
use blockifier::state::errors::StateError;
use cairo_felt::Felt252;
use camino::Utf8PathBuf;
use cheatnet::cheatcodes::{CheatcodeError, EnhancedHintError};
use cheatnet::constants::build_testing_state;
use cheatnet::forking::state::ForkStateReader;
use cheatnet::rpc::call_contract;
use cheatnet::state::ExtendedStateReader;
use cheatnet::CheatnetState;
use conversions::StarknetConversions;
use num_bigint::BigUint;
use starknet::core::types::BlockId;
use starknet_api::core::ContractAddress;
use std::str::FromStr;

#[test]
fn fork_simple() {
    let mut state = create_cheatnet_fork_state();

    let contract_address = Felt252::from(
        BigUint::from_str(
            "3216637956526895219277698311134811322769343974163380838558193911733621219342",
        )
        .unwrap(),
    )
    .to_contract_address();

    let selector = felt_selector_from_name("get_balance");
    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();
    assert_success!(output, vec![Felt252::from(2)]);

    let selector = felt_selector_from_name("increase_balance");
    call_contract(
        &contract_address,
        &selector,
        &[Felt252::from(100)],
        &mut state,
    )
    .unwrap();

    let selector = felt_selector_from_name("get_balance");
    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();
    assert_success!(output, vec![Felt252::from(102)]);
}

#[test]
fn try_calling_nonexistent_contract() {
    let mut state = create_cheatnet_fork_state();

    let contract_address = ContractAddress::from(1_u8);
    let selector = felt_selector_from_name("get_balance");

    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();
    assert_error!(
        output,
        "Contract not deployed at address: 0x0000000000000000000000000000000000000000000000000000000000000001"
    );
}

#[test]
fn try_deploying_undeclared_class() {
    let mut state = create_cheatnet_fork_state();

    let class_hash = "1".to_owned().to_class_hash();

    assert!(match state.deploy(&class_hash, &[]) {
        Err(CheatcodeError::Unrecoverable(EnhancedHintError::State(
            StateError::UndeclaredClassHash(undeclared_class_hash),
        ))) => undeclared_class_hash == class_hash,
        _ => false,
    });
}

#[test]
fn test_forking_at_block_number() {
    let predeployed_contracts = Utf8PathBuf::from("predeployed-contracts");
    let node_url =
        std::env::var("CHEATNET_RPC_URL").expect("CHEATNET_RPC_URL must be set in the .env file");

    let mut state = CheatnetState::new(ExtendedStateReader {
        dict_state_reader: build_testing_state(&predeployed_contracts),
        fork_state_reader: Some(ForkStateReader::new(&node_url, BlockId::Number(309_780))),
    });
    let higher_block_fork_id = state.add_fork(&node_url, BlockId::Number(309_781));

    let contract_address = Felt252::from(
        BigUint::from_str(
            "3216637956526895219277698311134811322769343974163380838558193911733621219342",
        )
        .unwrap(),
    )
    .to_contract_address();

    let selector = felt_selector_from_name("get_balance");
    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();
    assert_error!(
        output,
        "Contract not deployed at address: 0x071c8d74edc89330f314f3b1109059d68ebfa68874aa91e9c425a6378ffde00e"
    );

    state.set_current_fork_id(higher_block_fork_id).unwrap();
    state.blockifier_state = CachedState::from(state.blockifier_state.state);

    let selector = felt_selector_from_name("get_balance");
    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();
    assert_success!(output, vec![Felt252::from(0)]);
}

#[test]
fn call_forked_contract_from_other_contract() {
    let mut state = create_cheatnet_fork_state();

    let forked_contract_address = Felt252::from(
        BigUint::from_str(
            "3216637956526895219277698311134811322769343974163380838558193911733621219342",
        )
        .unwrap(),
    );

    let contract_address = deploy_contract(&mut state, "ForkingChecker", &[Felt252::from(1)]);

    let selector = felt_selector_from_name("get_balance_call_contract");
    let output = call_contract(
        &contract_address,
        &selector,
        &[forked_contract_address],
        &mut state,
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(2)]);
}

#[test]
fn library_call_on_forked_class_hash() {
    let mut state = create_cheatnet_fork_state();

    let forked_class_hash = Felt252::from(
        BigUint::from_str(
            "2721209982346623666255046859539202086457905975723689966720503254490557413774",
        )
        .unwrap(),
    );

    let contract_address = deploy_contract(&mut state, "ForkingChecker", &[Felt252::from(1)]);

    let selector = felt_selector_from_name("get_balance_library_call");
    let output = call_contract(
        &contract_address,
        &selector,
        &[forked_class_hash.clone()],
        &mut state,
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(0)]);

    call_contract(
        &contract_address,
        &felt_selector_from_name("set_balance"),
        &[Felt252::from(100)],
        &mut state,
    )
    .unwrap();

    let output = call_contract(
        &contract_address,
        &selector,
        &[forked_class_hash],
        &mut state,
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(100)]);
}

#[test]
fn call_forked_contract_from_constructor() {
    let mut state = create_cheatnet_fork_state();

    let forked_class_hash = Felt252::from(
        BigUint::from_str(
            "2721209982346623666255046859539202086457905975723689966720503254490557413774",
        )
        .unwrap(),
    );

    let forked_contract_address = Felt252::from(
        BigUint::from_str(
            "3216637956526895219277698311134811322769343974163380838558193911733621219342",
        )
        .unwrap(),
    );

    let contract_address = deploy_contract(
        &mut state,
        "ForkingChecker",
        &[Felt252::from(0), forked_contract_address],
    );

    let selector = felt_selector_from_name("get_balance_library_call");
    let output = call_contract(
        &contract_address,
        &selector,
        &[forked_class_hash.clone()],
        &mut state,
    )
    .unwrap();
    assert_success!(output, vec![Felt252::from(2)]);
}
