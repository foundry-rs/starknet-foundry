use crate::{
    assert_success,
    common::{deploy_contract, state::create_cheatnet_state},
};
use cairo_felt::Felt252;
use cheatnet::{conversions::felt_selector_from_name, rpc::call_contract};
use starknet_api::core::ContractAddress;

#[test]
fn prank_simple() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "PrankChecker", vec![].as_slice());

    state.start_prank(contract_address, ContractAddress::from(123_u128));

    let selector = felt_selector_from_name("get_caller_address");

    let output =
        call_contract(&contract_address, &selector, vec![].as_slice(), &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}
