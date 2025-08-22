use crate::common::assertions::assert_success;
use conversions::IntoConv;
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

use super::test_environment::TestEnvironment;

#[test]
fn meta_tx_v0_with_cheat_caller_address() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatCallerAddressCheckerMetaTxV0", &[]);

    test_env
        .cheatnet_state
        .start_cheat_caller_address(contract_address, ContractAddress::from(123_u32));

    let meta_contract = test_env.deploy("MetaTxV0Test", &[]);

    let signature = vec![];

    let meta_result = test_env.call_contract(
        &meta_contract,
        "execute_meta_tx_v0",
        &[contract_address.into_(), signature.len().into()]
            .into_iter()
            .chain(signature)
            .collect::<Vec<_>>(),
    );

    assert_success(meta_result, &[Felt::from(123)]);
}

#[test]
fn meta_tx_v0_with_cheat_block_number() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockNumberCheckerMetaTxV0", &[]);

    test_env
        .cheatnet_state
        .start_cheat_block_number(contract_address, 999_u64);

    let meta_contract = test_env.deploy("MetaTxV0Test", &[]);

    let signature = vec![];

    let meta_result = test_env.call_contract(
        &meta_contract,
        "execute_meta_tx_v0",
        &[contract_address.into_(), signature.len().into()]
            .into_iter()
            .chain(signature)
            .collect::<Vec<_>>(),
    );

    assert_success(meta_result, &[Felt::from(999)]);
}

#[test]
fn meta_tx_v0_with_cheat_block_timestamp() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockTimestampCheckerMetaTxV0", &[]);

    test_env
        .cheatnet_state
        .start_cheat_block_timestamp(contract_address, 1_234_567_890_u64);

    let meta_contract = test_env.deploy("MetaTxV0Test", &[]);

    let signature = vec![];

    let meta_result = test_env.call_contract(
        &meta_contract,
        "execute_meta_tx_v0",
        &[contract_address.into_(), signature.len().into()]
            .into_iter()
            .chain(signature)
            .collect::<Vec<_>>(),
    );

    assert_success(meta_result, &[Felt::from(1_234_567_890_u64)]);
}

#[test]
fn meta_tx_v0_with_cheat_sequencer_address() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatSequencerAddressCheckerMetaTxV0", &[]);

    test_env
        .cheatnet_state
        .start_cheat_sequencer_address(contract_address, ContractAddress::from(777_u32));

    let meta_contract = test_env.deploy("MetaTxV0Test", &[]);

    let signature = vec![];

    let meta_result = test_env.call_contract(
        &meta_contract,
        "execute_meta_tx_v0",
        &[contract_address.into_(), signature.len().into()]
            .into_iter()
            .chain(signature)
            .collect::<Vec<_>>(),
    );

    assert_success(meta_result, &[Felt::from(777)]);
}

#[test]
fn meta_tx_v0_with_cheat_block_hash() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatBlockHashCheckerMetaTxV0", &[]);

    let block_number = 100;

    test_env
        .cheatnet_state
        .start_cheat_block_hash(contract_address, block_number, Felt::from(555));

    let meta_contract = test_env.deploy("MetaTxV0Test", &[]);

    let signature = vec![];

    let meta_result = test_env.call_contract(
        &meta_contract,
        "execute_meta_tx_v0_get_block_hash",
        &[
            contract_address.into_(),
            block_number.into(),
            signature.len().into(),
        ]
        .into_iter()
        .chain(signature)
        .collect::<Vec<_>>(),
    );

    assert_success(meta_result, &[Felt::from(555)]);
}

#[test]
fn meta_tx_v0_verify_tx_context_modification() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("TxInfoCheckerMetaTxV0", &[]);

    // Call __execute__ directly, should return original version (3)
    let direct_execute_result = test_env.call_contract(&contract_address, "__execute__", &[]);

    let meta_contract = test_env.deploy("MetaTxV0Test", &[]);

    let signature = vec![];

    let meta_result = test_env.call_contract(
        &meta_contract,
        "execute_meta_tx_v0",
        &[contract_address.into_(), signature.len().into()]
            .into_iter()
            .chain(signature.clone())
            .collect::<Vec<_>>(),
    );

    assert_success(meta_result, &[Felt::ZERO]);

    assert_success(direct_execute_result, &[Felt::from(3_u32)]);
}
