use crate::common::state::build_runtime_state;
use crate::common::{call_contract, deploy_wrapper};
use crate::{
    assert_success,
    common::{
        deploy_contract, felt_selector_from_name, get_contracts, recover_data,
        state::create_cached_state,
    },
};
use blockifier::state::state_api::State;
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::call_to_blockifier_runtime_extension::RuntimeState;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::state::CheatnetState;
use starknet_api::core::ContractAddress;

// We've decided that the nonce should not change in tests
// and should remain 0 at all times, this may be revised in the future.
// For now to test nonce `spoof` should be used.

fn check_nonce(
    state: &mut dyn State,
    runtime_state: &mut RuntimeState,
    contract_address: &ContractAddress,
) -> Felt252 {
    let write_nonce = felt_selector_from_name("write_nonce");
    let read_nonce = felt_selector_from_name("read_nonce");

    let output = call_contract(state, runtime_state, contract_address, &write_nonce, &[]);

    assert_success!(output, vec![]);

    let output = call_contract(state, runtime_state, contract_address, &read_nonce, &[]);

    recover_data(output)[0].clone()
}

#[test]
fn nonce_transactions() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);

    let contract_address = deploy_contract(&mut cached_state, &mut runtime_state, "Noncer", &[]);

    let old_nonce = check_nonce(&mut cached_state, &mut runtime_state, &contract_address);
    let new_nonce = check_nonce(&mut cached_state, &mut runtime_state, &contract_address);

    assert_eq!(old_nonce, Felt252::from(0));
    assert_eq!(old_nonce, new_nonce);
}

#[test]
fn nonce_declare_deploy() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();
    let mut runtime_state = build_runtime_state(&mut cheatnet_state);
    let contract_address = deploy_contract(&mut cached_state, &mut runtime_state, "Noncer", &[]);

    let contracts = get_contracts();

    let nonce1 = check_nonce(&mut cached_state, &mut runtime_state, &contract_address);

    let class_hash = declare(&mut cached_state, "HelloStarknet", &contracts).unwrap();

    let nonce2 = check_nonce(&mut cached_state, &mut runtime_state, &contract_address);

    deploy_wrapper(&mut cached_state, &mut runtime_state, &class_hash, &[]).unwrap();

    let nonce3 = check_nonce(&mut cached_state, &mut runtime_state, &contract_address);

    assert_eq!(nonce1, Felt252::from(0));
    assert_eq!(nonce1, nonce2);
    assert_eq!(nonce2, nonce3);
}
