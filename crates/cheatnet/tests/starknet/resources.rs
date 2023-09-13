use std::collections::HashMap;

use crate::common::{
    deploy_contract, felt_selector_from_name, get_contracts, state::create_cheatnet_state,
};
use cairo_felt::Felt252;
use cheatnet::rpc::{call_contract, CallContractOutput, ResourceReport};
use conversions::StarknetConversions;

#[test]
fn call_resources_simple() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "HelloStarknet", &[]);

    let selector = felt_selector_from_name("increase_balance");

    let output = call_contract(
        &contract_address,
        &selector,
        &[Felt252::from(123)],
        &mut state,
    )
    .unwrap();

    assert!(match output {
        CallContractOutput::Success {
            resource_report, ..
        } => {
            resource_report
                == ResourceReport {
                    gas: 1.26,
                    steps: 126,
                    bultins: HashMap::from([("range_check_builtin".to_owned(), 2)]),
                }
        }
        _ => false,
    });
}

#[test]
fn deploy_resources_simple() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();

    let contract_name = "HelloStarknet".to_owned().to_felt252();
    let class_hash = state.declare(&contract_name, &contracts).unwrap();

    let payload = state.deploy(&class_hash, &[]).unwrap();

    assert!(
        payload.resource_report
            == ResourceReport {
                gas: 5.34,
                steps: 534,
                bultins: HashMap::from([("range_check_builtin".to_owned(), 15)]),
            }
    );
}
