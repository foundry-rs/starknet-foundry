use std::collections::HashMap;

use crate::common::{
    deploy_contract, felt_selector_from_name, get_contracts,
    state::{create_cached_state, create_cheatnet_state},
};
use cairo_felt::Felt252;
use cheatnet::cheatcodes::deploy::deploy;
use cheatnet::rpc::{call_contract, ResourceReport};
use conversions::StarknetConversions;

#[test]
fn call_resources_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "HelloStarknet",
        &[],
    );

    let selector = felt_selector_from_name("increase_balance");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        &[Felt252::from(123)],
    )
    .unwrap();

    assert_eq!(
        output.resource_report,
        ResourceReport {
            gas: 1.26,
            steps: 126,
            bultins: HashMap::from([("range_check_builtin".to_owned(), 2)]),
        }
    );
}

#[test]
fn deploy_resources_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();

    let contract_name = "HelloStarknet".to_owned().to_felt252();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let payload = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[]).unwrap();

    assert_eq!(
        payload.resource_report,
        ResourceReport {
            gas: 5.34,
            steps: 534,
            bultins: HashMap::from([("range_check_builtin".to_owned(), 15)]),
        }
    );
}
