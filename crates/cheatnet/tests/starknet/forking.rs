use crate::assert_success;
use crate::common::state::create_cheatnet_fork_state;
use cairo_felt::Felt252;
use cheatnet::conversions::{
    class_hash_from_felt, contract_address_from_felt, felt_selector_from_name,
};
use cheatnet::rpc::call_contract;
use num_bigint::BigUint;
use std::str::FromStr;

#[test]
fn fork_simple() {
    let mut state = create_cheatnet_fork_state();

    let contract_address = contract_address_from_felt(&Felt252::from(
        BigUint::from_str(
            "3216637956526895219277698311134811322769343974163380838558193911733621219342",
        )
        .unwrap(),
    ));

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
#[should_panic(expected = "Contract not found")]
fn try_calling_nonexistent_contract() {
    let mut state = create_cheatnet_fork_state();

    let contract_address = contract_address_from_felt(&Felt252::from(123));
    let selector = felt_selector_from_name("get_balance");

    call_contract(&contract_address, &selector, &[], &mut state).unwrap();
}

#[test]
#[should_panic(expected = "Class hash not found")]
fn try_deploying_undeclared_class() {
    let mut state = create_cheatnet_fork_state();

    let class_hash = class_hash_from_felt(&Felt252::from(123));

    state.deploy(&class_hash, &[]).unwrap();
}
