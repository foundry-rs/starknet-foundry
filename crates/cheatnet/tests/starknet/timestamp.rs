use crate::common::call_contract;
use crate::{
    assert_success,
    common::{
        deploy_contract, felt_selector_from_name, recover_data,
        state::{create_cached_state, create_cheatnet_state},
    },
};
use cairo_felt::Felt252;
use cheatnet::state::{BlockifierState, CheatnetState};
use starknet_api::core::ContractAddress;

fn check_timestamp(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
) -> Felt252 {
    let write_timestamp = felt_selector_from_name("write_timestamp");
    let read_timestamp = felt_selector_from_name("read_timestamp");

    let output = call_contract(
        blockifier_state,
        cheatnet_state,
        contract_address,
        &write_timestamp,
        &[],
    )
    .unwrap();

    assert_success!(output, vec![]);

    let output = call_contract(
        blockifier_state,
        cheatnet_state,
        contract_address,
        &read_timestamp,
        &[],
    )
    .unwrap();

    recover_data(output)[0].clone()
}

#[test]
fn timestamp_does_not_decrease() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "Timestamper",
        &[],
    );

    let old_timestamp = check_timestamp(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );
    let new_timestamp = check_timestamp(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    assert!(old_timestamp <= new_timestamp);
}
