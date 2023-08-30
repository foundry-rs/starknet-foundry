use crate::{
    assert_success,
    common::{deploy_contract, get_contracts, recover_data, state::create_cheatnet_state},
};
use cairo_felt::Felt252;
use cheatnet::{
    conversions::{felt_from_short_string, felt_selector_from_name},
    rpc::call_contract,
    CheatnetState,
};
use starknet_api::core::ContractAddress;

// We've decided that the nonce should not change in tests
// and should remain 0 at all times, this may be revised in the future.
// For now to test nonce `spoof` should be used.

fn check_nonce(state: &mut CheatnetState, contract_address: &ContractAddress) -> Felt252 {
    let write_nonce = felt_selector_from_name("write_nonce");
    let read_nonce = felt_selector_from_name("read_nonce");

    let output = call_contract(contract_address, &write_nonce, &[], state).unwrap();

    assert_success!(output, vec![]);

    let output = call_contract(contract_address, &read_nonce, &[], state).unwrap();

    recover_data(output)[0].clone()
}

#[test]
fn nonce_transactions() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "Noncer", &[]);

    let old_nonce = check_nonce(&mut state, &contract_address);
    let new_nonce = check_nonce(&mut state, &contract_address);

    assert_eq!(old_nonce, Felt252::from(0));
    assert_eq!(old_nonce, new_nonce);
}

#[test]
fn nonce_declare_deploy() {
    let mut state = create_cheatnet_state();
    let contract_address = deploy_contract(&mut state, "Noncer", &[]);

    let contracts = get_contracts();
    let contract_name = felt_from_short_string("HelloStarknet");

    let nonce1 = check_nonce(&mut state, &contract_address);

    let class_hash = state.declare(&contract_name, &contracts).unwrap();

    let nonce2 = check_nonce(&mut state, &contract_address);

    state.deploy(&class_hash, &[], None).unwrap();

    let nonce3 = check_nonce(&mut state, &contract_address);

    assert_eq!(nonce1, Felt252::from(0));
    assert_eq!(nonce1, nonce2);
    assert_eq!(nonce2, nonce3);
}
