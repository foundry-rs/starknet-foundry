use crate::{
    assert_success,
    common::{
        deploy_contract, felt_selector_from_name, recover_data,
        state::{create_cached_state, create_cheatnet_state},
    },
};
use cairo_felt::Felt252;
use cheatnet::{
    rpc::call_contract,
    state::{BlockifierState, CheatnetState},
};
use starknet_api::core::ContractAddress;

fn check_block(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
) -> (Felt252, Felt252, Felt252, Felt252) {
    let write_block = felt_selector_from_name("write_block");
    let read_block_number = felt_selector_from_name("read_block_number");
    let read_block_timestamp = felt_selector_from_name("read_block_timestamp");
    let read_sequencer_address = felt_selector_from_name("read_sequencer_address");
    let read_block_hash = felt_selector_from_name("read_block_hash");

    let output = call_contract(
        blockifier_state,
        cheatnet_state,
        contract_address,
        &write_block,
        &[],
    )
    .unwrap();

    assert_success!(output, vec![]);

    let output = call_contract(
        blockifier_state,
        cheatnet_state,
        contract_address,
        &read_block_number,
        &[],
    )
    .unwrap();

    let block_number = &recover_data(output)[0];

    let output = call_contract(
        blockifier_state,
        cheatnet_state,
        contract_address,
        &read_block_timestamp,
        &[],
    )
    .unwrap();

    let block_timestamp = &recover_data(output)[0];

    let output = call_contract(
        blockifier_state,
        cheatnet_state,
        contract_address,
        &read_sequencer_address,
        &[],
    )
    .unwrap();

    let sequencer_address = &recover_data(output)[0];

    let output = call_contract(
        blockifier_state,
        cheatnet_state,
        contract_address,
        &read_block_hash,
        &[],
    )
    .unwrap();

    let block_hash = &recover_data(output)[0];

    (
        block_number.clone(),
        block_timestamp.clone(),
        sequencer_address.clone(),
        block_hash.clone(),
    )
}

#[test]
fn block_does_not_decrease() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address =
        deploy_contract(&mut blockifier_state, &mut cheatnet_state, "Blocker", &[]);

    let (old_block_number, old_block_timestamp, old_sequencer_address, old_block_hash) =
        check_block(
            &mut blockifier_state,
            &mut cheatnet_state,
            &contract_address,
        );

    let (new_block_number, new_block_timestamp, new_sequencer_address, new_block_hash) =
        check_block(
            &mut blockifier_state,
            &mut cheatnet_state,
            &contract_address,
        );

    assert!(old_block_number <= new_block_number);
    assert!(old_block_timestamp <= new_block_timestamp);
    assert_eq!(old_sequencer_address, new_sequencer_address);
    assert_eq!(new_block_hash, old_block_hash);
}
