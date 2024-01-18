use crate::common::call_contract;
use crate::{
    assert_success,
    common::{
        call_contract_getter_by_name, deploy_contract, felt_selector_from_name, get_contracts,
        recover_data,
        state::{create_cached_state, create_cheatnet_state},
    },
};
use cairo_felt::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::{
    deploy::deploy, spoof::TxInfoMock,
};
use cheatnet::state::CheatTarget;
use cheatnet::state::{BlockifierState, CheatnetState};
use conversions::{felt252::FromShortString, IntoConv};
use num_traits::ToPrimitive;
use runtime::utils::BufferReader;
use starknet_api::core::ContractAddress;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct TxInfo {
    pub version: Felt252,
    pub account_contract_address: Felt252,
    pub max_fee: Felt252,
    pub signature: Vec<Felt252>,
    pub transaction_hash: Felt252,
    pub chain_id: Felt252,
    pub nonce: Felt252,
    pub resource_bounds: Vec<Felt252>,
    pub tip: Felt252,
    pub paymaster_data: Vec<Felt252>,
    pub nonce_data_availability_mode: Felt252,
    pub fee_data_availability_mode: Felt252,
    pub account_deployment_data: Vec<Felt252>,
}

impl TxInfo {
    fn apply_mock_fields(tx_info_mock: &TxInfoMock, tx_info: &Self) -> Self {
        macro_rules! clone_field {
            ($field:ident) => {
                tx_info_mock
                    .$field
                    .clone()
                    .unwrap_or(tx_info.$field.clone())
            };
        }

        Self {
            version: clone_field!(version),
            account_contract_address: clone_field!(account_contract_address),
            max_fee: clone_field!(max_fee),
            signature: clone_field!(signature),
            transaction_hash: clone_field!(transaction_hash),
            chain_id: clone_field!(chain_id),
            nonce: clone_field!(nonce),
            resource_bounds: clone_field!(resource_bounds),
            tip: clone_field!(tip),
            paymaster_data: clone_field!(paymaster_data),
            nonce_data_availability_mode: clone_field!(nonce_data_availability_mode),
            fee_data_availability_mode: clone_field!(fee_data_availability_mode),
            account_deployment_data: clone_field!(account_deployment_data),
        }
    }

    fn deserialize(data: &[Felt252]) -> Self {
        let mut reader = BufferReader::new(data);

        let version = reader.read_felt();
        let account_contract_address = reader.read_felt();
        let max_fee = reader.read_felt();
        let signature = reader.read_vec();
        let transaction_hash = reader.read_felt();
        let chain_id = reader.read_felt();
        let nonce = reader.read_felt();
        let resource_bounds_len = reader.read_felt();
        let resource_bounds = reader.read_vec_body(
            3 * resource_bounds_len.to_usize().unwrap(), // ResourceBounds struct has 3 fields
        );
        let tip = reader.read_felt();
        let paymaster_data = reader.read_vec();
        let nonce_data_availability_mode = reader.read_felt();
        let fee_data_availability_mode = reader.read_felt();
        let account_deployment_data = reader.read_vec();

        Self {
            version,
            account_contract_address,
            max_fee,
            signature,
            transaction_hash,
            chain_id,
            nonce,
            resource_bounds,
            tip,
            paymaster_data,
            nonce_data_availability_mode,
            fee_data_availability_mode,
            account_deployment_data,
        }
    }
}

fn assert_tx_info(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
    expected_tx_info: &TxInfo,
) {
    let tx_info = get_tx_info(blockifier_state, cheatnet_state, contract_address);
    assert_eq!(tx_info, expected_tx_info.to_owned());
}

fn get_tx_info(
    blockifier_state: &mut BlockifierState,
    cheatnet_state: &mut CheatnetState,
    contract_address: &ContractAddress,
) -> TxInfo {
    let get_tx_info_output = call_contract_getter_by_name(
        blockifier_state,
        cheatnet_state,
        contract_address,
        "get_tx_info",
    );
    let tx_info_data = recover_data(get_tx_info_output);
    TxInfo::deserialize(&tx_info_data)
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

    let tx_info_before = get_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    let tx_info_mock = TxInfoMock {
        transaction_hash: Some(Felt252::from(123)),
        ..Default::default()
    };

    let expected_tx_info = TxInfo::apply_mock_fields(&tx_info_mock, &tx_info_before);

    cheatnet_state.start_spoof(CheatTarget::One(contract_address), tx_info_mock);

    assert_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &expected_tx_info,
    );
}

#[test]
fn start_spoof_multiple_times() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpoofChecker",
        vec![].as_slice(),
    );

    let tx_info_before = get_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    let initial_tx_info_mock = TxInfoMock {
        version: Some(Felt252::from(13)),
        account_contract_address: Some(Felt252::from(66)),
        max_fee: Some(Felt252::from(77)),
        signature: Some(vec![Felt252::from(88), Felt252::from(89)]),
        transaction_hash: Some(Felt252::from(123)),
        chain_id: Some(Felt252::from(22)),
        nonce: Some(Felt252::from(33)),
        resource_bounds: Some(vec![
            Felt252::from(111),
            Felt252::from(222),
            Felt252::from(333),
            Felt252::from(444),
            Felt252::from(555),
            Felt252::from(666),
        ]),
        tip: Some(Felt252::from(777)),
        paymaster_data: Some(vec![
            Felt252::from(11),
            Felt252::from(22),
            Felt252::from(33),
            Felt252::from(44),
        ]),
        nonce_data_availability_mode: Some(Felt252::from(55)),
        fee_data_availability_mode: Some(Felt252::from(66)),
        account_deployment_data: Some(vec![
            Felt252::from(777),
            Felt252::from(888),
            Felt252::from(999),
        ]),
    };
    let expected_tx_info = TxInfo::apply_mock_fields(&initial_tx_info_mock, &tx_info_before);

    cheatnet_state.start_spoof(
        CheatTarget::One(contract_address),
        initial_tx_info_mock.clone(),
    );

    assert_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &expected_tx_info,
    );

    let tx_info_mock = TxInfoMock {
        version: None,
        max_fee: None,
        transaction_hash: None,
        nonce: None,
        tip: None,
        nonce_data_availability_mode: None,
        account_deployment_data: None,
        ..initial_tx_info_mock
    };
    let expected_tx_info = TxInfo::apply_mock_fields(&tx_info_mock, &tx_info_before);

    cheatnet_state.start_spoof(CheatTarget::One(contract_address), tx_info_mock);

    assert_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &expected_tx_info,
    );

    let tx_info_mock = TxInfoMock {
        account_contract_address: None,
        signature: None,
        chain_id: None,
        resource_bounds: None,
        paymaster_data: None,
        fee_data_availability_mode: None,
        ..initial_tx_info_mock
    };
    let expected_tx_info = TxInfo::apply_mock_fields(&tx_info_mock, &tx_info_before);

    cheatnet_state.start_spoof(CheatTarget::One(contract_address), tx_info_mock);

    assert_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &expected_tx_info,
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

    let tx_info_before = get_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    let tx_info_mock = TxInfoMock {
        transaction_hash: Some(Felt252::from(123)),
        ..Default::default()
    };

    let expected_tx_info = TxInfo::apply_mock_fields(&tx_info_mock, &tx_info_before);

    cheatnet_state.start_spoof(CheatTarget::One(contract_address), tx_info_mock);

    assert_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &expected_tx_info,
    );

    cheatnet_state.stop_spoof(CheatTarget::One(contract_address));

    assert_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &tx_info_before,
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

    let tx_info_before = get_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    cheatnet_state.stop_spoof(CheatTarget::One(contract_address));

    assert_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &tx_info_before,
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

    let tx_info_mock = TxInfoMock {
        transaction_hash: Some(Felt252::from(123)),
        ..Default::default()
    };

    cheatnet_state.start_spoof(CheatTarget::One(contract_address), tx_info_mock);

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

    let contract_name = Felt252::from_short_string("ConstructorSpoofChecker").unwrap();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();
    let precalculated_address = cheatnet_state.precalculate_address(&class_hash, vec![].as_slice());

    let tx_info_mock = TxInfoMock {
        transaction_hash: Some(Felt252::from(123)),
        ..Default::default()
    };

    cheatnet_state.start_spoof(CheatTarget::One(precalculated_address), tx_info_mock);

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

    let tx_info_mock = TxInfoMock {
        transaction_hash: Some(Felt252::from(123)),
        ..Default::default()
    };

    cheatnet_state.start_spoof(CheatTarget::One(contract_address), tx_info_mock);

    let selector = felt_selector_from_name("get_transaction_hash");

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
    let contract_name = Felt252::from_short_string("SpoofChecker").unwrap();
    let class_hash = blockifier_state
        .declare(&contract_name, &contracts)
        .unwrap();

    let lib_call_address = deploy_contract(
        &mut blockifier_state,
        &mut cheatnet_state,
        "SpoofCheckerLibCall",
        vec![].as_slice(),
    );

    let tx_info_mock = TxInfoMock {
        transaction_hash: Some(Felt252::from(123)),
        ..Default::default()
    };

    cheatnet_state.start_spoof(CheatTarget::One(lib_call_address), tx_info_mock);

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

    let tx_info_before = get_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    let tx_info_mock = TxInfoMock {
        transaction_hash: Some(Felt252::from(123)),
        ..Default::default()
    };

    let expected_tx_info = TxInfo::apply_mock_fields(&tx_info_mock, &tx_info_before);

    cheatnet_state.start_spoof(CheatTarget::All, tx_info_mock);

    assert_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &expected_tx_info,
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

    let tx_info_before = get_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    let mut tx_info_mock = TxInfoMock {
        transaction_hash: Some(Felt252::from(123)),
        ..Default::default()
    };

    cheatnet_state.start_spoof(CheatTarget::All, tx_info_mock.clone());

    tx_info_mock.transaction_hash = Some(Felt252::from(321));
    let expected_tx_info = TxInfo::apply_mock_fields(&tx_info_mock, &tx_info_before);

    cheatnet_state.start_spoof(CheatTarget::One(contract_address), tx_info_mock);

    assert_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &expected_tx_info,
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

    let tx_info_before = get_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    let mut tx_info_mock = TxInfoMock {
        transaction_hash: Some(Felt252::from(123)),
        ..Default::default()
    };

    cheatnet_state.start_spoof(CheatTarget::One(contract_address), tx_info_mock.clone());

    tx_info_mock.transaction_hash = Some(Felt252::from(321));
    let expected_tx_info = TxInfo::apply_mock_fields(&tx_info_mock, &tx_info_before);

    cheatnet_state.start_spoof(CheatTarget::All, tx_info_mock);

    assert_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &expected_tx_info,
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

    let tx_info_before = get_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
    );

    let tx_info_mock = TxInfoMock {
        transaction_hash: Some(Felt252::from(123)),
        ..Default::default()
    };

    cheatnet_state.start_spoof(CheatTarget::All, tx_info_mock);

    cheatnet_state.stop_spoof(CheatTarget::All);

    assert_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address,
        &tx_info_before,
    );
}

#[test]
fn spoof_multiple() {
    let mut cached_state = create_cached_state();
    let (mut blockifier_state, mut cheatnet_state) = create_cheatnet_state(&mut cached_state);

    let contract = Felt252::from_short_string("SpoofChecker").unwrap();
    let contracts = get_contracts();
    let class_hash = blockifier_state.declare(&contract, &contracts).unwrap();

    let contract_address_1 = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    let contract_address_2 = deploy(&mut blockifier_state, &mut cheatnet_state, &class_hash, &[])
        .unwrap()
        .contract_address;

    let tx_info_before_1 = get_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address_1,
    );
    let tx_info_before_2 = get_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address_2,
    );

    let tx_info_mock = TxInfoMock {
        transaction_hash: Some(Felt252::from(123)),
        ..Default::default()
    };
    let expected_tx_info_1 = TxInfo::apply_mock_fields(&tx_info_mock, &tx_info_before_1);
    let expected_tx_info_2 = TxInfo::apply_mock_fields(&tx_info_mock, &tx_info_before_2);

    cheatnet_state.start_spoof(
        CheatTarget::Multiple(vec![contract_address_1, contract_address_2]),
        tx_info_mock,
    );

    assert_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address_1,
        &expected_tx_info_1,
    );
    assert_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address_2,
        &expected_tx_info_2,
    );

    cheatnet_state.stop_spoof(CheatTarget::All);

    assert_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address_1,
        &tx_info_before_1,
    );
    assert_tx_info(
        &mut blockifier_state,
        &mut cheatnet_state,
        &contract_address_2,
        &tx_info_before_2,
    );
}
