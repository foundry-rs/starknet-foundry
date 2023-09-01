use crate::assert_success;
use crate::common::get_contracts;
use crate::common::state::create_cheatnet_state;
use cairo_felt::Felt252;
use cairo_vm::vm::errors::hint_errors::HintError;
use cheatnet::cheatcodes::CheatcodeError::Unrecoverable;
use cheatnet::cheatcodes::EnhancedHintError;
use cheatnet::conversions::{felt_from_short_string, felt_selector_from_name};
use cheatnet::rpc::call_contract;
use starknet_api::core::ContractAddress;
use starknet_api::transaction::ContractAddressSalt;

#[test]
fn deploy_at_predefined_address() {
    let mut state = create_cheatnet_state();

    let contract = felt_from_short_string("HelloStarknet");
    let contracts = get_contracts();

    let class_hash = state.declare(&contract, &contracts).unwrap();
    let contract_address = state
        .deploy_at(
            &class_hash,
            &[],
            ContractAddressSalt::default(),
            ContractAddress::from(1_u8),
        )
        .unwrap();

    assert_eq!(contract_address, ContractAddress::from(1_u8));

    let selector = felt_selector_from_name("get_balance");
    let output = call_contract(&contract_address, &selector, &[], &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(0)]);
}

#[test]
fn deploy_two_at_the_same_address() {
    let mut state = create_cheatnet_state();

    let contract = felt_from_short_string("HelloStarknet");
    let contracts = get_contracts();

    let class_hash = state.declare(&contract, &contracts).unwrap();
    state
        .deploy_at(
            &class_hash,
            &[],
            ContractAddressSalt::default(),
            ContractAddress::from(1_u8),
        )
        .unwrap();

    let result = state.deploy_at(
        &class_hash,
        &[],
        ContractAddressSalt::default(),
        ContractAddress::from(1_u8),
    );

    assert!(match result {
        Err(Unrecoverable(EnhancedHintError::Hint(HintError::CustomHint(err)))) =>
            err.as_ref() == "Address is already taken",
        _ => false,
    });
}
