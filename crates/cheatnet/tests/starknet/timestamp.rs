use crate::common::call_contract;
use crate::{
    common::assertions::assert_success,
    common::{deploy_contract, felt_selector_from_name, recover_data, state::create_cached_state},
};
use blockifier::state::state_api::State;
use cairo_felt::Felt252;
use cheatnet::state::CheatnetState;
use starknet_api::core::ContractAddress;

fn check_timestamp(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
) -> Felt252 {
    let write_timestamp = felt_selector_from_name("write_timestamp");
    let read_timestamp = felt_selector_from_name("read_timestamp");

    let output = call_contract(
        state,
        cheatnet_state,
        contract_address,
        &write_timestamp,
        &[],
    );

    assert_success(output, &[]);

    let output = call_contract(
        state,
        cheatnet_state,
        contract_address,
        &read_timestamp,
        &[],
    );
    recover_data(output)[0].clone()
}

#[test]
fn timestamp_does_not_decrease() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address =
        deploy_contract(&mut cached_state, &mut cheatnet_state, "Timestamper", &[]);

    let old_timestamp = check_timestamp(&mut cached_state, &mut cheatnet_state, &contract_address);
    let new_timestamp = check_timestamp(&mut cached_state, &mut cheatnet_state, &contract_address);

    assert!(old_timestamp <= new_timestamp);
}
