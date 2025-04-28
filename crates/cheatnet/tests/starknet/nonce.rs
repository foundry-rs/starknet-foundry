use crate::common::assertions::ClassHashAssert;
use crate::common::{call_contract, deploy_wrapper};
use crate::{
    common::assertions::assert_success,
    common::{deploy_contract, get_contracts, recover_data, state::create_cached_state},
};
use blockifier::state::state_api::State;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::declare::declare;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::storage::felt_selector_from_name;
use cheatnet::state::CheatnetState;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;
// We've decided that the nonce should not change in tests
// and should remain 0 at all times, this may be revised in the future.

fn check_nonce(
    state: &mut dyn State,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
) -> Felt {
    let write_nonce = felt_selector_from_name("write_nonce");
    let read_nonce = felt_selector_from_name("read_nonce");

    let output = call_contract(state, cheatnet_state, contract_address, write_nonce, &[]);

    assert_success(output, &[]);

    let output = call_contract(state, cheatnet_state, contract_address, read_nonce, &[]);

    recover_data(output)[0]
}

#[test]
fn nonce_transactions() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(&mut cached_state, &mut cheatnet_state, "Noncer", &[]);

    let old_nonce = check_nonce(&mut cached_state, &mut cheatnet_state, &contract_address);
    let new_nonce = check_nonce(&mut cached_state, &mut cheatnet_state, &contract_address);

    assert_eq!(old_nonce, Felt::from(0));
    assert_eq!(old_nonce, new_nonce);
}

#[test]
fn nonce_declare_deploy() {
    let mut cached_state = create_cached_state();
    let mut cheatnet_state = CheatnetState::default();

    let contract_address = deploy_contract(&mut cached_state, &mut cheatnet_state, "Noncer", &[]);

    let contracts_data = get_contracts();

    let nonce1 = check_nonce(&mut cached_state, &mut cheatnet_state, &contract_address);

    let class_hash = declare(&mut cached_state, "HelloStarknet", &contracts_data)
        .unwrap()
        .unwrap_success();

    let nonce2 = check_nonce(&mut cached_state, &mut cheatnet_state, &contract_address);

    deploy_wrapper(&mut cached_state, &mut cheatnet_state, &class_hash, &[]).unwrap();

    let nonce3 = check_nonce(&mut cached_state, &mut cheatnet_state, &contract_address);

    assert_eq!(nonce1, Felt::from(0));
    assert_eq!(nonce1, nonce2);
    assert_eq!(nonce2, nonce3);
}
