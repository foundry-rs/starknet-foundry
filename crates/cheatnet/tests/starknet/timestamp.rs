use crate::common::call_contract;
use crate::common::state::build_runtime_state;
use crate::{
    assert_success,
    common::{deploy_contract, felt_selector_from_name, recover_data, state::create_cached_state},
};
use blockifier::state::state_api::State;
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::RuntimeState;
use cheatnet::state::CheatnetState;
use starknet_api::core::ContractAddress;

fn check_timestamp(
    state: &mut dyn State,
    runtime_state: &mut RuntimeState,
    contract_address: &ContractAddress,
) -> Felt252 {
    let write_timestamp = felt_selector_from_name("write_timestamp");
    let read_timestamp = felt_selector_from_name("read_timestamp");

    let output = call_contract(
        state,
        runtime_state,
        contract_address,
        &write_timestamp,
        &[],
    );

    assert_success!(output, vec![]);

    let output = call_contract(state, runtime_state, contract_address, &read_timestamp, &[]);
    recover_data(output)[0].clone()
}

#[test]
fn timestamp_does_not_decrease() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address =
        deploy_contract(&mut cached_state, &mut runtime_state, "Timestamper", &[]);

    let old_timestamp = check_timestamp(&mut cached_state, &mut runtime_state, &contract_address);
    let new_timestamp = check_timestamp(&mut cached_state, &mut runtime_state, &contract_address);

    assert!(old_timestamp <= new_timestamp);
}
