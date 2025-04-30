use crate::common::call_contract;
use crate::common::{
    assertions::assert_success, deploy_contract, recover_data, state::create_cached_state,
};
use blockifier::state::state_api::State;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::selector_from_name;
use cheatnet::state::CheatnetState;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

fn check_block(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
) -> (Felt, Felt, Felt, Felt) {
    let write_block = selector_from_name("write_block");
    let read_block_number = selector_from_name("read_block_number");
    let read_block_timestamp = selector_from_name("read_block_timestamp");
    let read_sequencer_address = selector_from_name("read_sequencer_address");
    let read_block_hash = selector_from_name("read_block_hash");

    let output = call_contract(state, cheatnet_state, contract_address, write_block, &[]);

    assert_success(output, &[]);

    let output = call_contract(
        state,
        cheatnet_state,
        contract_address,
        read_block_number,
        &[],
    );

    let block_number = &recover_data(output)[0];

    let output = call_contract(
        state,
        cheatnet_state,
        contract_address,
        read_block_timestamp,
        &[],
    );

    let block_timestamp = &recover_data(output)[0];

    let output = call_contract(
        state,
        cheatnet_state,
        contract_address,
        read_sequencer_address,
        &[],
    );

    let sequencer_address = &recover_data(output)[0];

    let output = call_contract(
        state,
        cheatnet_state,
        contract_address,
        read_block_hash,
        &[],
    );

    let block_hash = &recover_data(output)[0];

    (
        *block_number,
        *block_timestamp,
        *sequencer_address,
        *block_hash,
    )
}

#[test]
fn block_does_not_decrease() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(&mut cached_state, &mut cheatnet_state, "Blocker", &[]);

    let (old_block_number, old_block_timestamp, old_sequencer_address, old_block_hash) =
        check_block(&mut cached_state, &mut cheatnet_state, &contract_address);

    let (new_block_number, new_block_timestamp, new_sequencer_address, new_block_hash) =
        check_block(&mut cached_state, &mut cheatnet_state, &contract_address);

    assert!(old_block_number <= new_block_number);
    assert!(old_block_timestamp <= new_block_timestamp);
    assert_eq!(old_sequencer_address, new_sequencer_address);
    assert_eq!(new_block_hash, old_block_hash);
}
