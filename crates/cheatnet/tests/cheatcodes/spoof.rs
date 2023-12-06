use crate::{
    assert_success,
    common::{
        call_contract_getter_by_name, deploy_contract, felt_selector_from_name, get_contracts,
        recover_data,
        state::{create_cached_state, create_cheatnet_state},
    },
};
use cairo_felt::Felt252;
use cheatnet::cheatcodes::deploy::deploy;
use cheatnet::state::CheatTarget;
use cheatnet::{
    rpc::call_contract,
    state::{BlockifierState, CheatnetState},
};
use conversions::IntoConv;
use starknet_api::core::ContractAddress;

#[allow(clippy::too_many_arguments)]
fn assert_all_mock_checker_getters(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
    expected_version: &[Felt252],
    expected_account_address: &[Felt252],
    expected_max_fee: &[Felt252],
    expected_signature: &[Felt252],
    expected_tx_hash: &[Felt252],
    expected_chain_id: &[Felt252],
    expected_nonce: &[Felt252],
    expected_resource_bounds: &[Felt252],
    expected_tip: &[Felt252],
    expected_paymaster_data: &[Felt252],
    expected_nonce_data_availabilty_mode: &[Felt252],
    expected_fee_data_availabilty_mode: &[Felt252],
    expected_account_deployment_data: &[Felt252],
) {
    let tx_hash = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_tx_hash",
    );
    assert_success!(tx_hash, expected_tx_hash);
    let nonce = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_nonce",
    );
    assert_success!(nonce, expected_nonce);
    let account_address = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_account_contract_address",
    );
    assert_success!(account_address, expected_account_address);
    let version = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_version",
    );
    assert_success!(version, expected_version);
    let chain_id = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_chain_id",
    );
    assert_success!(chain_id, expected_chain_id);
    let max_fee = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_max_fee",
    );
    assert_success!(max_fee, expected_max_fee);
    let signature = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_signature",
    );
    assert_success!(signature, expected_signature);
    let resource_bounds = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_resource_bounds",
    );
    assert_success!(resource_bounds, expected_resource_bounds);
    let tip = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_tip",
    );
    assert_success!(tip, expected_tip);
    let paymaster_data = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_paymaster_data",
    );
    assert_success!(paymaster_data, expected_paymaster_data);
    let nonce_data_availabilty_mode = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_nonce_data_availabilty_mode",
    );
    assert_success!(
        nonce_data_availabilty_mode,
        expected_nonce_data_availabilty_mode
    );
    let fee_data_availabilty_mode = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_fee_data_availabilty_mode",
    );
    assert_success!(
        fee_data_availabilty_mode,
        expected_fee_data_availabilty_mode
    );
    let account_deployment_data = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_account_deployment_data",
    );
    assert_success!(account_deployment_data, expected_account_deployment_data);
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_lines)]
fn call_mock_checker_getters(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
) -> (
    Vec<Felt252>,
    Vec<Felt252>,
    Vec<Felt252>,
    Vec<Felt252>,
    Vec<Felt252>,
    Vec<Felt252>,
    Vec<Felt252>,
    Vec<Felt252>,
    Vec<Felt252>,
    Vec<Felt252>,
    Vec<Felt252>,
    Vec<Felt252>,
    Vec<Felt252>,
) {
    let nonce = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_nonce",
    );
    let nonce = recover_data(nonce);
    let account_address = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_account_contract_address",
    );
    let account_address = recover_data(account_address);
    let version = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_version",
    );
    let version = recover_data(version);
    let chain_id = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_chain_id",
    );
    let chain_id = recover_data(chain_id);
    let max_fee = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_max_fee",
    );
    let max_fee = recover_data(max_fee);
    let signature = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_signature",
    );
    let signature = recover_data(signature);
    let tx_hash = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_tx_hash",
    );
    let tx_hash = recover_data(tx_hash);
    let resource_bounds = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_resource_bounds",
    );
    let resource_bounds = recover_data(resource_bounds);
    let tip = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_tip",
    );
    let tip = recover_data(tip);
    let paymaster_data = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_paymaster_data",
    );
    let paymaster_data = recover_data(paymaster_data);
    let nonce_data_availabilty_mode = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_nonce_data_availabilty_mode",
    );
    let nonce_data_availabilty_mode = recover_data(nonce_data_availabilty_mode);
    let fee_data_availabilty_mode = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_fee_data_availabilty_mode",
    );
    let fee_data_availabilty_mode = recover_data(fee_data_availabilty_mode);
    let account_deployment_data = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_account_deployment_data",
    );
    let account_deployment_data = recover_data(account_deployment_data);

    (
        nonce,
        account_address,
        version,
        chain_id,
        max_fee,
        signature,
        tx_hash,
        resource_bounds,
        tip,
        paymaster_data,
        nonce_data_availabilty_mode,
        fee_data_availabilty_mode,
        account_deployment_data,
    )
}

#[test]
fn spoof_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpoofChecker",
        vec![].as_slice(),
    );

    let (
        nonce_before,
        account_address_before,
        version_before,
        chain_id_before,
        max_fee_before,
        signature_before,
        _,
        resource_bounds_before,
        tip_before,
        paymaster_data_before,
        nonce_data_availabilty_mode_before,
        fee_data_availabilty_mode_before,
        account_deployment_data_before,
    ) = call_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    cheatnet_state.start_spoof(
        CheatTarget::One(contract_address),
        None,
        None,
        None,
        None,
        Some(Felt252::from(123)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert_all_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &version_before,
        &account_address_before,
        &max_fee_before,
        &signature_before,
        &[Felt252::from(123)],
        &chain_id_before,
        &nonce_before,
        &resource_bounds_before,
        &tip_before,
        &paymaster_data_before,
        &nonce_data_availabilty_mode_before,
        &fee_data_availabilty_mode_before,
        &account_deployment_data_before,
    );
}

#[test]
#[allow(clippy::too_many_lines)]
fn start_spoof_multiple_times() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpoofChecker",
        vec![].as_slice(),
    );

    let (
        nonce_before,
        account_address_before,
        version_before,
        chain_id_before,
        max_fee_before,
        signature_before,
        tx_hash_before,
        resource_bounds_before,
        tip_before,
        paymaster_data_before,
        nonce_data_availabilty_mode_before,
        fee_data_availabilty_mode_before,
        account_deployment_data_before,
    ) = call_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    let expected_version = Felt252::from(13);
    let expected_account_address = Felt252::from(66);
    let expected_max_fee = Felt252::from(77);
    let expected_signature = vec![Felt252::from(88), Felt252::from(89)];
    let expected_tx_hash = Felt252::from(123);
    let expected_chain_id = Felt252::from(22);
    let expected_nonce = Felt252::from(33);
    let expected_resource_bounds = vec![
        Felt252::from(111),
        Felt252::from(222),
        Felt252::from(333),
        Felt252::from(444),
        Felt252::from(555),
        Felt252::from(666),
    ];
    let expected_tip = Felt252::from(777);
    let expected_paymaster_data = vec![
        Felt252::from(11),
        Felt252::from(22),
        Felt252::from(33),
        Felt252::from(44),
    ];
    let expected_nonce_data_availabilty_mode = Felt252::from(55);
    let expected_fee_data_availabilty_mode = Felt252::from(66);
    let expected_account_deployment_data =
        vec![Felt252::from(777), Felt252::from(888), Felt252::from(999)];

    cheatnet_state.start_spoof(
        CheatTarget::One(contract_address),
        Some(expected_version.clone()),
        Some(expected_account_address.clone()),
        Some(expected_max_fee.clone()),
        Some(expected_signature.clone()),
        Some(expected_tx_hash.clone()),
        Some(expected_chain_id.clone()),
        Some(expected_nonce.clone()),
        Some(expected_resource_bounds.clone()),
        Some(expected_tip.clone()),
        Some(expected_paymaster_data.clone()),
        Some(expected_nonce_data_availabilty_mode.clone()),
        Some(expected_fee_data_availabilty_mode.clone()),
        Some(expected_account_deployment_data.clone()),
    );

    assert_all_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &[expected_version.clone()],
        &[expected_account_address.clone()],
        &[expected_max_fee.clone()],
        &[vec![Felt252::from(2)], expected_signature.clone()].concat(),
        &[expected_tx_hash.clone()],
        &[expected_chain_id.clone()],
        &[expected_nonce.clone()],
        &[vec![Felt252::from(2)], expected_resource_bounds.clone()].concat(),
        &[expected_tip.clone()],
        &[vec![Felt252::from(4)], expected_paymaster_data.clone()].concat(),
        &[expected_nonce_data_availabilty_mode.clone()],
        &[expected_fee_data_availabilty_mode.clone()],
        &[
            vec![Felt252::from(3)],
            expected_account_deployment_data.clone(),
        ]
        .concat(),
    );

    cheatnet_state.start_spoof(
        CheatTarget::One(contract_address),
        None,
        Some(expected_account_address.clone()),
        None,
        Some(expected_signature.clone()),
        None,
        Some(expected_chain_id.clone()),
        None,
        Some(expected_resource_bounds.clone()),
        None,
        Some(expected_paymaster_data.clone()),
        None,
        Some(expected_fee_data_availabilty_mode.clone()),
        None,
    );

    assert_all_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &version_before,
        &[expected_account_address],
        &max_fee_before,
        &[vec![Felt252::from(2)], expected_signature].concat(),
        &tx_hash_before,
        &[expected_chain_id],
        &nonce_before,
        &[vec![Felt252::from(2)], expected_resource_bounds].concat(),
        &tip_before,
        &[vec![Felt252::from(4)], expected_paymaster_data].concat(),
        &nonce_data_availabilty_mode_before,
        &[expected_fee_data_availabilty_mode],
        &account_deployment_data_before,
    );

    cheatnet_state.start_spoof(
        CheatTarget::One(contract_address),
        Some(expected_version.clone()),
        None,
        Some(expected_max_fee.clone()),
        None,
        Some(expected_tx_hash.clone()),
        None,
        Some(expected_nonce.clone()),
        None,
        Some(expected_tip.clone()),
        None,
        Some(expected_nonce_data_availabilty_mode.clone()),
        None,
        Some(expected_account_deployment_data.clone()),
    );

    assert_all_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &[expected_version],
        &account_address_before,
        &[expected_max_fee],
        &signature_before,
        &[expected_tx_hash],
        &chain_id_before,
        &[expected_nonce],
        &resource_bounds_before,
        &[expected_tip],
        &paymaster_data_before,
        &[expected_nonce_data_availabilty_mode],
        &fee_data_availabilty_mode_before,
        &[vec![Felt252::from(3)], expected_account_deployment_data].concat(),
    );
}

#[test]
fn spoof_start_stop() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpoofChecker",
        vec![].as_slice(),
    );

    let (
        nonce_before,
        account_address_before,
        version_before,
        chain_id_before,
        max_fee_before,
        signature_before,
        tx_hash_before,
        resource_bounds_before,
        tip_before,
        paymaster_data_before,
        nonce_data_availabilty_mode_before,
        fee_data_availabilty_mode_before,
        account_deployment_data_before,
    ) = call_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    cheatnet_state.start_spoof(
        CheatTarget::One(contract_address),
        None,
        None,
        None,
        None,
        Some(Felt252::from(123)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert_all_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &version_before,
        &account_address_before,
        &max_fee_before,
        &signature_before,
        &[Felt252::from(123)],
        &chain_id_before,
        &nonce_before,
        &resource_bounds_before,
        &tip_before,
        &paymaster_data_before,
        &nonce_data_availabilty_mode_before,
        &fee_data_availabilty_mode_before,
        &account_deployment_data_before,
    );

    cheatnet_state.stop_spoof(CheatTarget::One(contract_address));

    assert_all_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &version_before,
        &account_address_before,
        &max_fee_before,
        &signature_before,
        &tx_hash_before,
        &chain_id_before,
        &nonce_before,
        &resource_bounds_before,
        &tip_before,
        &paymaster_data_before,
        &nonce_data_availabilty_mode_before,
        &fee_data_availabilty_mode_before,
        &account_deployment_data_before,
    );
}

#[test]
fn spoof_stop_no_effect() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpoofChecker",
        vec![].as_slice(),
    );

    let (
        nonce_before,
        account_address_before,
        version_before,
        chain_id_before,
        max_fee_before,
        signature_before,
        tx_hash_before,
        resource_bounds_before,
        tip_before,
        paymaster_data_before,
        nonce_data_availabilty_mode_before,
        fee_data_availabilty_mode_before,
        account_deployment_data_before,
    ) = call_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    cheatnet_state.stop_spoof(CheatTarget::One(contract_address));

    assert_all_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &version_before,
        &account_address_before,
        &max_fee_before,
        &signature_before,
        &tx_hash_before,
        &chain_id_before,
        &nonce_before,
        &resource_bounds_before,
        &tip_before,
        &paymaster_data_before,
        &nonce_data_availabilty_mode_before,
        &fee_data_availabilty_mode_before,
        &account_deployment_data_before,
    );
}

#[test]
fn spoof_with_other_syscall() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpoofChecker",
        vec![].as_slice(),
    );

    cheatnet_state.start_spoof(
        CheatTarget::One(contract_address),
        None,
        None,
        None,
        None,
        Some(Felt252::from(123)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let selector = felt_selector_from_name("get_tx_hash_and_emit_event");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        vec![].as_slice(),
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn spoof_in_constructor() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();

    let contract_name = "ConstructorSpoofChecker".to_owned().into_();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();
    let precalculated_address = cheatnet_state.precalculate_address(&class_hash, vec![].as_slice());

    cheatnet_state.start_spoof(
        CheatTarget::One(precalculated_address),
        None,
        None,
        None,
        None,
        Some(Felt252::from(123)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let contract_address = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    assert_eq!(precalculated_address, contract_address);

    let selector = felt_selector_from_name("get_stored_tx_hash");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        vec![].as_slice(),
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn spoof_proxy() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpoofChecker",
        vec![].as_slice(),
    );

    cheatnet_state.start_spoof(
        CheatTarget::One(contract_address),
        None,
        None,
        None,
        None,
        Some(Felt252::from(123)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let selector = felt_selector_from_name("get_tx_hash");

    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &selector,
        vec![].as_slice(),
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);

    let proxy_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpoofCheckerProxy",
        vec![].as_slice(),
    );
    let proxy_selector = felt_selector_from_name("get_spoof_checkers_tx_hash");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &proxy_address,
        &proxy_selector,
        &[contract_address.into_()],
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn spoof_library_call() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contracts = get_contracts();
    let contract_name = "SpoofChecker".to_owned().into_();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let lib_call_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpoofCheckerLibCall",
        vec![].as_slice(),
    );

    cheatnet_state.start_spoof(
        CheatTarget::One(lib_call_address),
        None,
        None,
        None,
        None,
        Some(Felt252::from(123)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    let lib_call_selector = felt_selector_from_name("get_tx_hash_with_lib_call");
    let output = call_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        &lib_call_address,
        &lib_call_selector,
        &[class_hash.into_()],
    )
    .unwrap();

    assert_success!(output, vec![Felt252::from(123)]);
}

#[test]
fn spoof_all_simple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpoofChecker",
        &[],
    );

    let (
        nonce_before,
        account_address_before,
        version_before,
        chain_id_before,
        max_fee_before,
        signature_before,
        _,
        resource_bounds_before,
        tip_before,
        paymaster_data_before,
        nonce_data_availabilty_mode_before,
        fee_data_availabilty_mode_before,
        account_deployment_data_before,
    ) = call_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    cheatnet_state.start_spoof(
        CheatTarget::All,
        None,
        None,
        None,
        None,
        Some(Felt252::from(123)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert_all_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &version_before,
        &account_address_before,
        &max_fee_before,
        &signature_before,
        &[Felt252::from(123)],
        &chain_id_before,
        &nonce_before,
        &resource_bounds_before,
        &tip_before,
        &paymaster_data_before,
        &nonce_data_availabilty_mode_before,
        &fee_data_availabilty_mode_before,
        &account_deployment_data_before,
    );
}

#[test]
fn spoof_all_then_one() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpoofChecker",
        &[],
    );

    let (
        nonce_before,
        account_address_before,
        version_before,
        chain_id_before,
        max_fee_before,
        signature_before,
        _,
        resource_bounds_before,
        tip_before,
        paymaster_data_before,
        nonce_data_availabilty_mode_before,
        fee_data_availabilty_mode_before,
        account_deployment_data_before,
    ) = call_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    cheatnet_state.start_spoof(
        CheatTarget::All,
        None,
        None,
        None,
        None,
        Some(Felt252::from(123)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    cheatnet_state.start_spoof(
        CheatTarget::One(contract_address),
        None,
        None,
        None,
        None,
        Some(Felt252::from(321)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert_all_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &version_before,
        &account_address_before,
        &max_fee_before,
        &signature_before,
        &[Felt252::from(321)],
        &chain_id_before,
        &nonce_before,
        &resource_bounds_before,
        &tip_before,
        &paymaster_data_before,
        &nonce_data_availabilty_mode_before,
        &fee_data_availabilty_mode_before,
        &account_deployment_data_before,
    );
}

#[test]
fn spoof_one_then_all() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpoofChecker",
        &[],
    );

    let (
        nonce_before,
        account_address_before,
        version_before,
        chain_id_before,
        max_fee_before,
        signature_before,
        _,
        resource_bounds_before,
        tip_before,
        paymaster_data_before,
        nonce_data_availabilty_mode_before,
        fee_data_availabilty_mode_before,
        account_deployment_data_before,
    ) = call_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    cheatnet_state.start_spoof(
        CheatTarget::One(contract_address),
        None,
        None,
        None,
        None,
        Some(Felt252::from(123)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    cheatnet_state.start_spoof(
        CheatTarget::All,
        None,
        None,
        None,
        None,
        Some(Felt252::from(321)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    assert_all_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &version_before,
        &account_address_before,
        &max_fee_before,
        &signature_before,
        &[Felt252::from(321)],
        &chain_id_before,
        &nonce_before,
        &resource_bounds_before,
        &tip_before,
        &paymaster_data_before,
        &nonce_data_availabilty_mode_before,
        &fee_data_availabilty_mode_before,
        &account_deployment_data_before,
    );
}

#[test]
fn spoof_all_stop() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpoofChecker",
        &[],
    );

    let (
        nonce_before,
        account_address_before,
        version_before,
        chain_id_before,
        max_fee_before,
        signature_before,
        txn_hash_before,
        resource_bounds_before,
        tip_before,
        paymaster_data_before,
        nonce_data_availabilty_mode_before,
        fee_data_availabilty_mode_before,
        account_deployment_data_before,
    ) = call_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    cheatnet_state.start_spoof(
        CheatTarget::All,
        None,
        None,
        None,
        None,
        Some(Felt252::from(123)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );

    cheatnet_state.stop_spoof(CheatTarget::All);

    assert_all_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &version_before,
        &account_address_before,
        &max_fee_before,
        &signature_before,
        &txn_hash_before,
        &chain_id_before,
        &nonce_before,
        &resource_bounds_before,
        &tip_before,
        &paymaster_data_before,
        &nonce_data_availabilty_mode_before,
        &fee_data_availabilty_mode_before,
        &account_deployment_data_before,
    );
}

#[allow(clippy::too_many_lines)]
#[test]
fn spoof_multiple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = "SpoofChecker".to_owned().into_();
    let contracts = get_contracts();
    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let contract_address_1 = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    let contract_address_2 = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;
    let (
        nonce_before_1,
        account_address_before_1,
        version_before_1,
        chain_id_before_1,
        max_fee_before_1,
        signature_before_1,
        txn_hash_before_1,
        resource_bounds_before_1,
        tip_before_1,
        paymaster_data_before_1,
        nonce_data_availabilty_mode_before_1,
        fee_data_availabilty_mode_before_1,
        account_deployment_data_before_1,
    ) = call_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address_1,
    );
    let (
        nonce_before_2,
        account_address_before_2,
        version_before_2,
        chain_id_before_2,
        max_fee_before_2,
        signature_before_2,
        txn_hash_before_2,
        resource_bounds_before_2,
        tip_before_2,
        paymaster_data_before_2,
        nonce_data_availabilty_mode_before_2,
        fee_data_availabilty_mode_before_2,
        account_deployment_data_before_2,
    ) = call_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address_2,
    );

    cheatnet_state.start_spoof(
        CheatTarget::Multiple(vec![contract_address_1, contract_address_2]),
        None,
        None,
        None,
        None,
        Some(Felt252::from(123)),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert_all_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address_1,
        &version_before_1,
        &account_address_before_1,
        &max_fee_before_1,
        &signature_before_1,
        &[Felt252::from(123)],
        &chain_id_before_1,
        &nonce_before_1,
        &resource_bounds_before_1,
        &tip_before_1,
        &paymaster_data_before_1,
        &nonce_data_availabilty_mode_before_1,
        &fee_data_availabilty_mode_before_1,
        &account_deployment_data_before_1,
    );
    assert_all_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address_2,
        &version_before_2,
        &account_address_before_2,
        &max_fee_before_2,
        &signature_before_2,
        &[Felt252::from(123)],
        &chain_id_before_2,
        &nonce_before_2,
        &resource_bounds_before_2,
        &tip_before_2,
        &paymaster_data_before_2,
        &nonce_data_availabilty_mode_before_2,
        &fee_data_availabilty_mode_before_2,
        &account_deployment_data_before_2,
    );
    cheatnet_state.stop_spoof(CheatTarget::All);

    assert_all_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address_1,
        &version_before_1,
        &account_address_before_1,
        &max_fee_before_1,
        &signature_before_1,
        &txn_hash_before_1,
        &chain_id_before_1,
        &nonce_before_1,
        &resource_bounds_before_1,
        &tip_before_1,
        &paymaster_data_before_1,
        &nonce_data_availabilty_mode_before_1,
        &fee_data_availabilty_mode_before_1,
        &account_deployment_data_before_1,
    );
    assert_all_mock_checker_getters(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address_2,
        &version_before_2,
        &account_address_before_2,
        &max_fee_before_2,
        &signature_before_2,
        &txn_hash_before_2,
        &chain_id_before_2,
        &nonce_before_2,
        &resource_bounds_before_2,
        &tip_before_2,
        &paymaster_data_before_2,
        &nonce_data_availabilty_mode_before_2,
        &fee_data_availabilty_mode_before_2,
        &account_deployment_data_before_2,
    );
}
