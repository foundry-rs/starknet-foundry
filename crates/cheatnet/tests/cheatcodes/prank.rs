use crate::common::{create_cheatnet_state, deploy_contract};
use cairo_felt::Felt252;
use cheatnet::rpc::call_contract;
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::ContractAddress;

#[test]
fn prank_simple() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "PrankChecker", vec![].as_slice());

    state.start_prank(contract_address, ContractAddress::from(123_u128));

    let selector = get_selector_from_name("get_caller_address").unwrap();
    let selector = Felt252::from_bytes_be(&selector.to_bytes_be());

    let output =
        call_contract(&contract_address, &selector, vec![].as_slice(), &mut state).unwrap();

    println!("h");
}
