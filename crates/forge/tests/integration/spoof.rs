use indoc::indoc;
use std::path::Path;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_passed, test_case};

#[test]
fn start_and_stop_spoof_single_attribute() {
    let test = test_case!(
        indoc!(
            r"
            use result::ResultTrait;
            use box::BoxTrait;
            use starknet::info::TxInfo;
            use serde::Serde;
            use starknet::ContractAddress;
            use array::SpanTrait;
            use snforge_std::{ declare, ContractClassTrait, start_spoof, stop_spoof, TxInfoMock, TxInfoMockTrait, CheatTarget };
            use starknet::info::v2::ResourceBounds;

            #[starknet::interface]
            trait ISpoofChecker<TContractState> {
                fn get_tx_info(ref self: TContractState) -> starknet::info::v2::TxInfo;
            }

            #[test]
            fn start_spoof_single_attribute() {
                let contract = declare('SpoofChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpoofCheckerDispatcher { contract_address };

                let tx_info_before = dispatcher.get_tx_info();

                let mut tx_info_mock = TxInfoMockTrait::default();
                tx_info_mock.transaction_hash = Option::Some(421);

                start_spoof(CheatTarget::One(contract_address), tx_info_mock);

                let mut expected_tx_info = tx_info_before;
                expected_tx_info.transaction_hash = 421;

                assert_tx_info(dispatcher.get_tx_info(), expected_tx_info);

                stop_spoof(CheatTarget::One(contract_address));

                assert_tx_info(dispatcher.get_tx_info(), tx_info_before);
            }

            #[test]
            fn test_spoof_all_stop_one() {
                let contract = declare('SpoofChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpoofCheckerDispatcher { contract_address };

                let tx_info_before = dispatcher.get_tx_info();

                let mut tx_info_mock = TxInfoMockTrait::default();
                tx_info_mock.transaction_hash = Option::Some(421);

                start_spoof(CheatTarget::All, tx_info_mock);

                let mut expected_tx_info = tx_info_before;
                expected_tx_info.transaction_hash = 421;

                assert_tx_info(dispatcher.get_tx_info(), expected_tx_info);

                stop_spoof(CheatTarget::One(contract_address));

                assert_tx_info(dispatcher.get_tx_info(), tx_info_before);
            }

            fn assert_tx_info(tx_info: starknet::info::v2::TxInfo, expected_tx_info: starknet::info::v2::TxInfo) {
                assert(tx_info.version == expected_tx_info.version, 'Invalid version');
                assert(tx_info.account_contract_address == expected_tx_info.account_contract_address, 'Invalid account_contract_addr');
                assert(tx_info.max_fee == expected_tx_info.max_fee, 'Invalid max_fee');
                assert(tx_info.signature == expected_tx_info.signature, 'Invalid signature');
                assert(tx_info.transaction_hash == expected_tx_info.transaction_hash, 'Invalid transaction_hash');
                assert(tx_info.chain_id == expected_tx_info.chain_id, 'Invalid chain_id');
                assert(tx_info.nonce == expected_tx_info.nonce, 'Invalid nonce');

                let mut resource_bounds = array![];
                tx_info.resource_bounds.serialize(ref resource_bounds);

                let mut expected_resource_bounds = array![];
                expected_tx_info.resource_bounds.serialize(ref expected_resource_bounds);

                assert(resource_bounds == expected_resource_bounds, 'Invalid resource bounds');
                
                assert(tx_info.tip == expected_tx_info.tip, 'Invalid tip');
                assert(tx_info.paymaster_data == expected_tx_info.paymaster_data, 'Invalid paymaster_data');
                assert(tx_info.nonce_data_availability_mode == expected_tx_info.nonce_data_availability_mode, 'Invalid nonce_data_av_mode');
                assert(tx_info.fee_data_availability_mode == expected_tx_info.fee_data_availability_mode, 'Invalid fee_data_av_mode');
                assert(tx_info.account_deployment_data == expected_tx_info.account_deployment_data, 'Invalid account_deployment_data');
            }
        "
        ),
        Contract::from_code_path(
            "SpoofChecker".to_string(),
            Path::new("tests/data/contracts/spoof_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn start_spoof_all_attributes_mocked() {
    let test = test_case!(
        indoc!(
            r"
            use result::ResultTrait;
            use option::OptionTrait;
            use starknet::info::TxInfo;
            use serde::Serde;
            use traits::Into;
            use starknet::ContractAddress;
            use starknet::ContractAddressIntoFelt252;
            use starknet::Felt252TryIntoContractAddress;
            use array::SpanTrait;
            use snforge_std::{ declare, ContractClassTrait, start_spoof, stop_spoof, TxInfoMock, TxInfoMockTrait, CheatTarget };
            use starknet::info::v2::ResourceBounds;

            #[starknet::interface]
            trait ISpoofChecker<TContractState> {
                fn get_tx_hash(ref self: TContractState) -> felt252;
                fn get_nonce(ref self: TContractState) -> felt252;
                fn get_account_contract_address(ref self: TContractState) -> ContractAddress;
                fn get_signature(ref self: TContractState) -> Span<felt252>;
                fn get_version(ref self: TContractState) -> felt252;
                fn get_max_fee(ref self: TContractState) -> u128;
                fn get_chain_id(ref self: TContractState) -> felt252;
                fn get_resource_bounds(ref self: TContractState) -> Span<ResourceBounds>;
                fn get_tip(ref self: TContractState) -> u128;
                fn get_paymaster_data(ref self: TContractState) -> Span<felt252>;
                fn get_nonce_data_availability_mode(ref self: TContractState) -> u32;
                fn get_fee_data_availability_mode(ref self: TContractState) -> u32;
                fn get_account_deployment_data(ref self: TContractState) -> Span<felt252>;
            }

            #[test]
            fn start_spoof_all_attributes_mocked() {
                let contract = declare('SpoofChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpoofCheckerDispatcher { contract_address };

                let mut tx_info_mock = TxInfoMockTrait::default();
                tx_info_mock.nonce = Option::Some(411);
                tx_info_mock.account_contract_address = Option::Some(422.try_into().unwrap());
                tx_info_mock.version = Option::Some(433);
                tx_info_mock.transaction_hash = Option::Some(444);
                tx_info_mock.chain_id = Option::Some(455);
                tx_info_mock.max_fee = Option::Some(466_u128);
                tx_info_mock.signature = Option::Some(array![477, 478].span());
                tx_info_mock.resource_bounds = Option::Some(array![ResourceBounds { resource: 55, max_amount: 66, max_price_per_unit: 77 }, ResourceBounds { resource: 111, max_amount: 222, max_price_per_unit: 333 }].span());
                tx_info_mock.tip = Option::Some(123);
                tx_info_mock.paymaster_data = Option::Some(array![22, 33, 44].span());
                tx_info_mock.nonce_data_availability_mode = Option::Some(99);
                tx_info_mock.fee_data_availability_mode = Option::Some(88);
                tx_info_mock.account_deployment_data = Option::Some(array![111, 222].span());

                start_spoof(CheatTarget::One(contract_address), tx_info_mock);

                let nonce = dispatcher.get_nonce();
                assert(nonce == 411, 'Invalid nonce');

                let account_contract_address: felt252 = dispatcher.get_account_contract_address().into();
                assert(account_contract_address == 422, 'Invalid account address');

                let version = dispatcher.get_version();
                assert(version == 433, 'Invalid version');

                let transaction_hash = dispatcher.get_tx_hash();
                assert(transaction_hash == 444, 'Invalid tx hash');

                let chain_id = dispatcher.get_chain_id();
                assert(chain_id == 455, 'Invalid chain_id');

                let max_fee = dispatcher.get_max_fee();
                assert(max_fee == 466_u128, 'Invalid max_fee');

                let signature = dispatcher.get_signature();
                assert(signature.len() == 2, 'Invalid signature len');
                assert(*signature.at(0) == 477, 'Invalid signature el[0]');
                assert(*signature.at(1) == 478, 'Invalid signature el[1]');

                let resource_bounds = dispatcher.get_resource_bounds();
                assert(resource_bounds.len() == 2, 'Invalid resource_bounds len');
                assert(*resource_bounds.at(0).resource == 55, 'Invalid resource_bounds[0][0]');
                assert(*resource_bounds.at(0).max_amount == 66, 'Invalid resource_bounds[0][1]');
                assert(*resource_bounds.at(0).max_price_per_unit == 77, 'Invalid resource_bounds[0][2]');
                assert(*resource_bounds.at(1).resource == 111, 'Invalid resource_bounds[1][0]');
                assert(*resource_bounds.at(1).max_amount == 222, 'Invalid resource_bounds[1][1]');
                assert(*resource_bounds.at(1).max_price_per_unit == 333, 'Invalid resource_bounds[1][2]');

                let tip = dispatcher.get_tip();
                assert(tip == 123, 'Invalid tip');

                let paymaster_data = dispatcher.get_paymaster_data();
                assert(paymaster_data == array![22, 33, 44].span(), 'Invalid paymaster_data');

                let nonce_data_availability_mode = dispatcher.get_nonce_data_availability_mode();
                assert(nonce_data_availability_mode == 99, 'Invalid nonce data');

                let fee_data_availability_mode = dispatcher.get_fee_data_availability_mode();
                assert(fee_data_availability_mode == 88, 'Invalid fee data');

                let account_deployment_data = dispatcher.get_account_deployment_data();
                assert(account_deployment_data == array![111, 222].span(), 'Invalid account deployment');
            }
        "
        ),
        Contract::from_code_path(
            "SpoofChecker".to_string(),
            Path::new("tests/data/contracts/spoof_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn start_spoof_cancel_mock_by_setting_attribute_to_none() {
    let test = test_case!(
        indoc!(
            r"
            use result::ResultTrait;
            use box::BoxTrait;
            use starknet::info::TxInfo;
            use serde::Serde;
            use starknet::ContractAddress;
            use array::SpanTrait;
            use snforge_std::{ declare, ContractClassTrait, start_spoof, stop_spoof, TxInfoMock, TxInfoMockTrait, CheatTarget };
            use starknet::info::v2::ResourceBounds;

            #[starknet::interface]
            trait ISpoofChecker<TContractState> {
                fn get_tx_info(ref self: TContractState) -> starknet::info::v2::TxInfo;
            }

            #[test]
            fn start_spoof_cancel_mock_by_setting_attribute_to_none() {
                let contract = declare('SpoofChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpoofCheckerDispatcher { contract_address };

                let tx_info_before_mock = dispatcher.get_tx_info();

                let mut tx_info_mock = TxInfoMockTrait::default();
                tx_info_mock.transaction_hash = Option::Some(421);

                start_spoof(CheatTarget::One(contract_address), tx_info_mock);

                let mut expected_tx_info = tx_info_before_mock;
                expected_tx_info.transaction_hash = 421;

                assert_tx_info(dispatcher.get_tx_info(), expected_tx_info);

                start_spoof(CheatTarget::One(contract_address), TxInfoMockTrait::default());

                assert_tx_info(dispatcher.get_tx_info(), tx_info_before_mock);
            }

            fn assert_tx_info(tx_info: starknet::info::v2::TxInfo, expected_tx_info: starknet::info::v2::TxInfo) {
                assert(tx_info.version == expected_tx_info.version, 'Invalid version');
                assert(tx_info.account_contract_address == expected_tx_info.account_contract_address, 'Invalid account_contract_addr');
                assert(tx_info.max_fee == expected_tx_info.max_fee, 'Invalid max_fee');
                assert(tx_info.signature == expected_tx_info.signature, 'Invalid signature');
                assert(tx_info.transaction_hash == expected_tx_info.transaction_hash, 'Invalid transaction_hash');
                assert(tx_info.chain_id == expected_tx_info.chain_id, 'Invalid chain_id');
                assert(tx_info.nonce == expected_tx_info.nonce, 'Invalid nonce');

                let mut resource_bounds = array![];
                tx_info.resource_bounds.serialize(ref resource_bounds);

                let mut expected_resource_bounds = array![];
                expected_tx_info.resource_bounds.serialize(ref expected_resource_bounds);

                assert(resource_bounds == expected_resource_bounds, 'Invalid resource bounds');
                
                assert(tx_info.tip == expected_tx_info.tip, 'Invalid tip');
                assert(tx_info.paymaster_data == expected_tx_info.paymaster_data, 'Invalid paymaster_data');
                assert(tx_info.nonce_data_availability_mode == expected_tx_info.nonce_data_availability_mode, 'Invalid nonce_data_av_mode');
                assert(tx_info.fee_data_availability_mode == expected_tx_info.fee_data_availability_mode, 'Invalid fee_data_av_mode');
                assert(tx_info.account_deployment_data == expected_tx_info.account_deployment_data, 'Invalid account_deployment_data');
            }
        "
        ),
        Contract::from_code_path(
            "SpoofChecker".to_string(),
            Path::new("tests/data/contracts/spoof_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn start_spoof_no_attributes_mocked() {
    let test = test_case!(
        indoc!(
            r"
            use result::ResultTrait;
            use option::OptionTrait;
            use starknet::info::TxInfo;
            use serde::Serde;
            use traits::Into;
            use starknet::ContractAddress;
            use starknet::ContractAddressIntoFelt252;
            use array::SpanTrait;
            use snforge_std::{ declare, ContractClassTrait, start_spoof, stop_spoof, TxInfoMock, TxInfoMockTrait, CheatTarget };
            use starknet::info::v2::ResourceBounds;

            #[starknet::interface]
            trait ISpoofChecker<TContractState> {
                fn get_tx_info(ref self: TContractState) -> starknet::info::v2::TxInfo;
            }

            #[test]
            fn start_spoof_no_attributes_mocked() {
                let contract = declare('SpoofChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpoofCheckerDispatcher { contract_address };

                let tx_info_before_mock = dispatcher.get_tx_info();

                let mut tx_info_mock = TxInfoMockTrait::default();
                start_spoof(CheatTarget::One(contract_address), tx_info_mock);

                assert_tx_info(dispatcher.get_tx_info(), tx_info_before_mock);
            }

            fn assert_tx_info(tx_info: starknet::info::v2::TxInfo, expected_tx_info: starknet::info::v2::TxInfo) {
                assert(tx_info.version == expected_tx_info.version, 'Invalid version');
                assert(tx_info.account_contract_address == expected_tx_info.account_contract_address, 'Invalid account_contract_addr');
                assert(tx_info.max_fee == expected_tx_info.max_fee, 'Invalid max_fee');
                assert(tx_info.signature == expected_tx_info.signature, 'Invalid signature');
                assert(tx_info.transaction_hash == expected_tx_info.transaction_hash, 'Invalid transaction_hash');
                assert(tx_info.chain_id == expected_tx_info.chain_id, 'Invalid chain_id');
                assert(tx_info.nonce == expected_tx_info.nonce, 'Invalid nonce');

                let mut resource_bounds = array![];
                tx_info.resource_bounds.serialize(ref resource_bounds);

                let mut expected_resource_bounds = array![];
                expected_tx_info.resource_bounds.serialize(ref expected_resource_bounds);

                assert(resource_bounds == expected_resource_bounds, 'Invalid resource bounds');
                
                assert(tx_info.tip == expected_tx_info.tip, 'Invalid tip');
                assert(tx_info.paymaster_data == expected_tx_info.paymaster_data, 'Invalid paymaster_data');
                assert(tx_info.nonce_data_availability_mode == expected_tx_info.nonce_data_availability_mode, 'Invalid nonce_data_av_mode');
                assert(tx_info.fee_data_availability_mode == expected_tx_info.fee_data_availability_mode, 'Invalid fee_data_av_mode');
                assert(tx_info.account_deployment_data == expected_tx_info.account_deployment_data, 'Invalid account_deployment_data');
            }
        "
        ),
        Contract::from_code_path(
            "SpoofChecker".to_string(),
            Path::new("tests/data/contracts/spoof_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn start_spoof_multiple() {
    let test = test_case!(
        indoc!(
            r"
            use result::ResultTrait;
            use option::OptionTrait;
            use starknet::info::TxInfo;
            use serde::Serde;
            use traits::Into;
            use starknet::ContractAddress;
            use starknet::ContractAddressIntoFelt252;
            use array::SpanTrait;
            use snforge_std::{ declare, ContractClassTrait, start_spoof, TxInfoMock, TxInfoMockTrait, CheatTarget };

            #[starknet::interface]
            trait ISpoofChecker<TContractState> {
                fn get_tx_hash(ref self: TContractState) -> felt252;
            }

            #[test]
            fn start_spoof_multiple() {
                let contract = declare('SpoofChecker');
                
                let contract_address_1 = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher_1 = ISpoofCheckerDispatcher { contract_address: contract_address_1 };
                
                let contract_address_2 = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher_2 = ISpoofCheckerDispatcher { contract_address: contract_address_2 };

                let mut tx_info_mock = TxInfoMockTrait::default();
                tx_info_mock.transaction_hash = Option::Some(421);
                
                start_spoof(
                    CheatTarget::Multiple(array![contract_address_1, contract_address_2]),
                    tx_info_mock
                );

                let transaction_hash = dispatcher_1.get_tx_hash();
                assert(transaction_hash == 421, 'Invalid tx hash');
                
                let transaction_hash = dispatcher_2.get_tx_hash();
                assert(transaction_hash == 421, 'Invalid tx hash');
            }
        "
        ),
        Contract::from_code_path(
            "SpoofChecker".to_string(),
            Path::new("tests/data/contracts/spoof_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
#[allow(clippy::too_many_lines)]
fn start_spoof_all() {
    let test = test_case!(
        indoc!(
            r"
            use result::ResultTrait;
            use option::OptionTrait;
            use starknet::info::TxInfo;
            use serde::Serde;
            use traits::Into;
            use starknet::ContractAddress;
            use starknet::ContractAddressIntoFelt252;
            use array::SpanTrait;
            use snforge_std::{ declare, ContractClassTrait, start_spoof, stop_spoof, TxInfoMock, TxInfoMockTrait, CheatTarget };
            use starknet::info::v2::ResourceBounds;

            #[starknet::interface]
            trait ISpoofChecker<TContractState> {
                fn get_tx_hash(ref self: TContractState) -> felt252;
                fn get_nonce(ref self: TContractState) -> felt252;
                fn get_account_contract_address(ref self: TContractState) -> ContractAddress;
                fn get_signature(ref self: TContractState) -> Span<felt252>;
                fn get_version(ref self: TContractState) -> felt252;
                fn get_max_fee(ref self: TContractState) -> u128;
                fn get_chain_id(ref self: TContractState) -> felt252;
                fn get_resource_bounds(ref self: TContractState) -> Span<ResourceBounds>;
                fn get_tip(ref self: TContractState) -> u128;
                fn get_paymaster_data(ref self: TContractState) -> Span<felt252>;
                fn get_nonce_data_availability_mode(ref self: TContractState) -> u32;
                fn get_fee_data_availability_mode(ref self: TContractState) -> u32;
                fn get_account_deployment_data(ref self: TContractState) -> Span<felt252>;
            }

            #[test]
            fn start_spoof_all_one_param() {
                let contract = declare('SpoofChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpoofCheckerDispatcher { contract_address };

                let mut tx_info_mock = TxInfoMockTrait::default();
                tx_info_mock.transaction_hash = Option::Some(421);
                start_spoof(CheatTarget::All, tx_info_mock);

                let transaction_hash = dispatcher.get_tx_hash();
                assert(transaction_hash == 421, 'Invalid tx hash');
            }
            
            #[test]
            fn start_spoof_all_multiple_params() {
                let contract = declare('SpoofChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpoofCheckerDispatcher { contract_address };

                let mut tx_info_mock = TxInfoMockTrait::default();
                tx_info_mock.nonce = Option::Some(411);
                tx_info_mock.account_contract_address = Option::Some(422.try_into().unwrap());
                tx_info_mock.version = Option::Some(433);
                tx_info_mock.transaction_hash = Option::Some(444);
                tx_info_mock.chain_id = Option::Some(455);
                tx_info_mock.max_fee = Option::Some(466_u128);
                tx_info_mock.signature = Option::Some(array![477, 478].span());
                tx_info_mock.resource_bounds = Option::Some(array![ResourceBounds { resource: 55, max_amount: 66, max_price_per_unit: 77 }, ResourceBounds { resource: 111, max_amount: 222, max_price_per_unit: 333 }].span());
                tx_info_mock.tip = Option::Some(123);
                tx_info_mock.paymaster_data = Option::Some(array![22, 33, 44].span());
                tx_info_mock.nonce_data_availability_mode = Option::Some(99);
                tx_info_mock.fee_data_availability_mode = Option::Some(88);
                tx_info_mock.account_deployment_data = Option::Some(array![111, 222].span());

                start_spoof(CheatTarget::All, tx_info_mock);

                let nonce = dispatcher.get_nonce();
                assert(nonce == 411, 'Invalid nonce');

                let account_contract_address: felt252 = dispatcher.get_account_contract_address().into();
                assert(account_contract_address == 422, 'Invalid account address');

                let version = dispatcher.get_version();
                assert(version == 433, 'Invalid version');

                let transaction_hash = dispatcher.get_tx_hash();
                assert(transaction_hash == 444, 'Invalid tx hash');

                let chain_id = dispatcher.get_chain_id();
                assert(chain_id == 455, 'Invalid chain_id');

                let max_fee = dispatcher.get_max_fee();
                assert(max_fee == 466_u128, 'Invalid max_fee');

                let signature = dispatcher.get_signature();
                assert(signature.len() == 2, 'Invalid signature len');
                assert(*signature.at(0) == 477, 'Invalid signature el[0]');
                assert(*signature.at(1) == 478, 'Invalid signature el[1]');

                let resource_bounds = dispatcher.get_resource_bounds();
                assert(resource_bounds.len() == 2, 'Invalid resource_bounds len');
                assert(*resource_bounds.at(0).resource == 55, 'Invalid resource_bounds[0][0]');
                assert(*resource_bounds.at(0).max_amount == 66, 'Invalid resource_bounds[0][1]');
                assert(*resource_bounds.at(0).max_price_per_unit == 77, 'Invalid resource_bounds[0][2]');
                assert(*resource_bounds.at(1).resource == 111, 'Invalid resource_bounds[1][0]');
                assert(*resource_bounds.at(1).max_amount == 222, 'Invalid resource_bounds[1][1]');
                assert(*resource_bounds.at(1).max_price_per_unit == 333, 'Invalid resource_bounds[1][2]');

                let tip = dispatcher.get_tip();
                assert(tip == 123, 'Invalid tip');

                let paymaster_data = dispatcher.get_paymaster_data();
                assert(paymaster_data == array![22, 33, 44].span(), 'Invalid paymaster_data');

                let nonce_data_availability_mode = dispatcher.get_nonce_data_availability_mode();
                assert(nonce_data_availability_mode == 99, 'Invalid nonce data');

                let fee_data_availability_mode = dispatcher.get_fee_data_availability_mode();
                assert(fee_data_availability_mode == 88, 'Invalid fee data');

                let account_deployment_data = dispatcher.get_account_deployment_data();
                assert(account_deployment_data == array![111, 222].span(), 'Invalid account deployment');
            }
        "
        ),
        Contract::from_code_path(
            "SpoofChecker".to_string(),
            Path::new("tests/data/contracts/spoof_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn start_spoof_complex() {
    let test = test_case!(
        indoc!(
            r"
            use result::ResultTrait;
            use option::OptionTrait;
            use starknet::info::TxInfo;
            use serde::Serde;
            use traits::Into;
            use starknet::ContractAddress;
            use starknet::ContractAddressIntoFelt252;
            use array::SpanTrait;
            use snforge_std::{ declare, ContractClassTrait, start_spoof, TxInfoMock, TxInfoMockTrait, CheatTarget };

            #[starknet::interface]
            trait ISpoofChecker<TContractState> {
                fn get_tx_hash(ref self: TContractState) -> felt252;
            }

            #[test]
            fn start_spoof_complex() {
                let contract = declare('SpoofChecker');
                let contract_address_1 = contract.deploy(@array![]).unwrap();
                let contract_address_2 = contract.deploy(@array![]).unwrap();
                
                let dispatcher_1 = ISpoofCheckerDispatcher { contract_address: contract_address_1 };
                let dispatcher_2 = ISpoofCheckerDispatcher { contract_address: contract_address_2 };

                let mut tx_info_mock = TxInfoMockTrait::default();
                tx_info_mock.transaction_hash = Option::Some(421);
                start_spoof(CheatTarget::All, tx_info_mock);

                let transaction_hash_1 = dispatcher_1.get_tx_hash();
                let transaction_hash_2 = dispatcher_2.get_tx_hash();
                assert(transaction_hash_1 == 421, 'Invalid tx hash');
                assert(transaction_hash_2 == 421, 'Invalid tx hash');
                
                tx_info_mock.transaction_hash = Option::Some(621);
                start_spoof(CheatTarget::One(contract_address_2), tx_info_mock);
                
                let transaction_hash_1 = dispatcher_1.get_tx_hash();
                let transaction_hash_2 = dispatcher_2.get_tx_hash();
                assert(transaction_hash_1 == 421, 'Invalid tx hash');
                assert(transaction_hash_2 == 621, 'Invalid tx hash');
                
                tx_info_mock.transaction_hash = Option::Some(821);
                start_spoof(CheatTarget::Multiple(array![contract_address_1, contract_address_2]), tx_info_mock);
                
                let transaction_hash_1 = dispatcher_1.get_tx_hash();
                let transaction_hash_2 = dispatcher_2.get_tx_hash();
                assert(transaction_hash_1 == 821, 'Invalid tx hash');
                assert(transaction_hash_2 == 821, 'Invalid tx hash');
            }
        "
        ),
        Contract::from_code_path(
            "SpoofChecker".to_string(),
            Path::new("tests/data/contracts/spoof_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}
