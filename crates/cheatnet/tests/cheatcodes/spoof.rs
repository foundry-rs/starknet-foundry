use crate::{
    assert_success,
    common::{deploy_contract, get_contracts, recover_data, state::create_cheatnet_state},
};
use cairo_felt::Felt252;
use cheatnet::{
    conversions::{
        class_hash_to_felt, contract_address_to_felt, felt_from_short_string,
        felt_selector_from_name,
    },
    rpc::call_contract,
};

#[test]
fn spoof_simple() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "SpoofChecker", vec![].as_slice());

    let version = None;
    let account_contract_address = None;
    let max_fee = None;
    let signature = None;
    let transaction_hash = Some(Felt252::from(123));
    let chain_id = None;
    let nonce = None;

    state.start_spoof(
        contract_address,
        version,
        account_contract_address,
        max_fee,
        signature,
        transaction_hash,
        chain_id,
        nonce,
    );

    let selector_tx_hash = felt_selector_from_name("get_tx_hash");
    let tx_hash = call_contract(
        &contract_address,
        &selector_tx_hash,
        vec![].as_slice(),
        &mut state,
    )
    .unwrap();

    assert_success!(tx_hash, vec![Felt252::from(123)]);

    let selector_nonce = felt_selector_from_name("get_nonce");
    let chain_id = call_contract(
        &contract_address,
        &selector_nonce,
        vec![].as_slice(),
        &mut state,
    )
    .unwrap();

    assert_success!(chain_id, vec![Felt252::from(0)]);
}

#[test]
fn spoof_start_stop() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "SpoofChecker", vec![].as_slice());

    let selector_tx_hash = felt_selector_from_name("get_tx_hash");
    let tx_hash_before = call_contract(
        &contract_address,
        &selector_tx_hash,
        vec![].as_slice(),
        &mut state,
    )
    .unwrap();

    let tx_hash_before = recover_data(tx_hash_before);

    let version = None;
    let account_contract_address = None;
    let max_fee = None;
    let signature = None;
    let transaction_hash = Some(Felt252::from(123));
    let chain_id = None;
    let nonce = None;

    state.start_spoof(
        contract_address,
        version,
        account_contract_address,
        max_fee,
        signature,
        transaction_hash,
        chain_id,
        nonce,
    );

    let tx_hash = call_contract(
        &contract_address,
        &selector_tx_hash,
        vec![].as_slice(),
        &mut state,
    )
    .unwrap();

    assert_success!(tx_hash, vec![Felt252::from(123)]);

    state.stop_spoof(contract_address);

    let tx_hash = call_contract(
        &contract_address,
        &selector_tx_hash,
        vec![].as_slice(),
        &mut state,
    )
    .unwrap();

    assert_success!(tx_hash, tx_hash_before);
}

#[test]
fn spoof_stop_no_effect() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "SpoofChecker", vec![].as_slice());

    let selector_tx_hash = felt_selector_from_name("get_tx_hash");
    let tx_hash_before = call_contract(
        &contract_address,
        &selector_tx_hash,
        vec![].as_slice(),
        &mut state,
    )
    .unwrap();

    let tx_hash_before = recover_data(tx_hash_before);
    state.stop_spoof(contract_address);

    let tx_hash = call_contract(
        &contract_address,
        &selector_tx_hash,
        vec![].as_slice(),
        &mut state,
    )
    .unwrap();

    assert_success!(tx_hash, tx_hash_before);
}

#[test]
fn spoof_with_other_syscall() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "SpoofChecker", vec![].as_slice());

    let version = None;
    let account_contract_address = None;
    let max_fee = None;
    let signature = None;
    let transaction_hash = Some(Felt252::from(123));
    let chain_id = None;
    let nonce = None;

    state.start_spoof(
        contract_address,
        version,
        account_contract_address,
        max_fee,
        signature,
        transaction_hash,
        chain_id,
        nonce,
    );

    let selector = felt_selector_from_name("get_tx_hash_and_emit_event");

    let output =
        call_contract(&contract_address, &selector, vec![].as_slice(), &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
#[ignore = "TODO(#254)"]
fn spoof_in_constructor() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();

    let contract_name = felt_from_short_string("ConstructorSpoofChecker");
    let class_hash = state.declare(&contract_name, &contracts).unwrap();
    let precalculated_address = state.precalculate_address(&class_hash, vec![].as_slice());

    let version = None;
    let account_contract_address = None;
    let max_fee = None;
    let signature = None;
    let transaction_hash = Some(Felt252::from(123));
    let chain_id = None;
    let nonce = None;

    state.start_spoof(
        precalculated_address,
        version,
        account_contract_address,
        max_fee,
        signature,
        transaction_hash,
        chain_id,
        nonce,
    );

    let contract_address = state.deploy(&class_hash, vec![].as_slice()).unwrap();

    assert_eq!(precalculated_address, contract_address);

    let selector = felt_selector_from_name("get_stored_tx_hash");

    let output =
        call_contract(&contract_address, &selector, vec![].as_slice(), &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn spoof_proxy() {
    let mut state = create_cheatnet_state();

    let contract_address = deploy_contract(&mut state, "SpoofChecker", vec![].as_slice());

    let version = None;
    let account_contract_address = None;
    let max_fee = None;
    let signature = None;
    let transaction_hash = Some(Felt252::from(123));
    let chain_id = None;
    let nonce = None;

    state.start_spoof(
        contract_address,
        version,
        account_contract_address,
        max_fee,
        signature,
        transaction_hash,
        chain_id,
        nonce,
    );

    let selector = felt_selector_from_name("get_tx_hash");

    let output =
        call_contract(&contract_address, &selector, vec![].as_slice(), &mut state).unwrap();

    assert_success!(output, vec![Felt252::from(123)]);

    let proxy_address = deploy_contract(&mut state, "SpoofCheckerProxy", vec![].as_slice());
    let proxy_selector = felt_selector_from_name("get_spoof_checkers_tx_hash");
    let output = call_contract(
        &proxy_address,
        &proxy_selector,
        vec![contract_address_to_felt(contract_address)].as_slice(),
        &mut state,
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn spoof_library_call() {
    let mut state = create_cheatnet_state();

    let contracts = get_contracts();
    let contract_name = felt_from_short_string("SpoofChecker");
    let class_hash = state.declare(&contract_name, &contracts).unwrap();

    let lib_call_address = deploy_contract(&mut state, "SpoofCheckerLibCall", vec![].as_slice());

    let version = None;
    let account_contract_address = None;
    let max_fee = None;
    let signature = None;
    let transaction_hash = Some(Felt252::from(123));
    let chain_id = None;
    let nonce = None;

    state.start_spoof(
        lib_call_address,
        version,
        account_contract_address,
        max_fee,
        signature,
        transaction_hash,
        chain_id,
        nonce,
    );

    let lib_call_selector = felt_selector_from_name("get_tx_hash_with_lib_call");
    let output = call_contract(
        &lib_call_address,
        &lib_call_selector,
        vec![class_hash_to_felt(class_hash)].as_slice(),
        &mut state,
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}
