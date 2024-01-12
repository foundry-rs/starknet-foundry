use crate::common::call_contract;
use crate::{
    assert_success,
    common::{
        deploy_contract, felt_selector_from_name, get_contracts, recover_data,
        state::{create_cached_state, create_cheatnet_state},
    },
};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::deploy::deploy;
use cheatnet::state::{BlockifierState, CheatnetState};
use conversions::felt252::FromShortString;
use starknet_api::core::ContractAddress;

// We've decided that the nonce should not change in tests
// and should remain 0 at all times, this may be revised in the future.
// For now to test nonce `spoof` should be used.

fn check_nonce(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
) -> Felt252 {
    let write_nonce = felt_selector_from_name("write_nonce");
    let read_nonce = felt_selector_from_name("read_nonce");

    let output = call_contract(
        blockifier_state,
        cheatnet_state,
        contract_address,
        &write_nonce,
        &[],
    )
    .unwrap();

    assert_success!(output, vec![]);

    let output = call_contract(
        blockifier_state,
        cheatnet_state,
        contract_address,
        &read_nonce,
        &[],
    )
    .unwrap();

    recover_data(output)[0].clone()
}

#[test]
fn nonce_transactions() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address =
        deploy_contract(&mut blockifier_state, &mut cheatnet_state, "Noncer", &[]);

    let old_nonce = check_nonce(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );
    let new_nonce = check_nonce(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    assert_eq!(old_nonce, Felt252::from(0));
    assert_eq!(old_nonce, new_nonce);
}

#[test]
fn nonce_declare_deploy() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);
    let contract_address =
        deploy_contract(&mut blockifier_state, &mut cheatnet_state, "Noncer", &[]);

    let contracts = get_contracts();
    let contract_name = Felt252::from_short_string("HelloStarknet").unwrap();

    let nonce1 = check_nonce(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let nonce2 = check_nonce(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[]).unwrap();

    let nonce3 = check_nonce(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    assert_eq!(nonce1, Felt252::from(0));
    assert_eq!(nonce1, nonce2);
    assert_eq!(nonce2, nonce3);
}
