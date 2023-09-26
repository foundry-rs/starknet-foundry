use crate::{
    assert_success,
    common::{
        deploy_contract, felt_selector_from_name, recover_data, state::create_cheatnet_state,
    },
};
use cairo_felt::Felt252;
use cheatnet::{rpc::call_contract, CheatnetState};
use starknet_api::core::ContractAddress;

fn check_timestamp(state: &mut CheatnetState, contract_address: &ContractAddress) -> Felt252 {
    let write_timestamp = felt_selector_from_name("write_timestamp");
    let read_timestamp = felt_selector_from_name("read_timestamp");

    let output = call_contract(contract_address, &write_timestamp, &[], state).unwrap();

    assert_success!(output, vec![]);

    let output = call_contract(contract_address, &read_timestamp, &[], state).unwrap();

    recover_data(output)[0].clone()
}

#[test]
fn timestamp_does_not_decrease() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "Timestamper", &[]);

    let old_timestamp = check_timestamp(&mut state, &contract_address);
    let new_timestamp = check_timestamp(&mut state, &contract_address);

    assert!(old_timestamp <= new_timestamp);
}
