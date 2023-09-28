use indoc::indoc;
use std::path::Path;
use utils::runner::Contract;
use utils::running_tests::run_test_case;
use utils::{assert_passed, test_case};

#[test]
fn start_and_stop_spoof_single_attribute() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use box::BoxTrait;
            use starknet::info::TxInfo;
            use serde::Serde;
            use starknet::ContractAddress;
            use array::SpanTrait;
            use snforge_std::{ declare, ContractClassTrait, start_spoof, stop_spoof, TxInfoMock, TxInfoMockTrait };

            #[starknet::interface]
            trait ISpoofChecker<TContractState> {
                fn get_tx_hash(ref self: TContractState) -> felt252;
                fn get_nonce(ref self: TContractState) -> felt252;
                fn get_account_contract_address(ref self: TContractState) -> ContractAddress;
                fn get_signature(ref self: TContractState) -> Span<felt252>;
                fn get_version(ref self: TContractState) -> felt252;
                fn get_max_fee(ref self: TContractState) -> u128;
                fn get_chain_id(ref self: TContractState) -> felt252;
            }

            #[test]
            fn start_spoof_single_attribute() {
                let contract = declare('SpoofChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpoofCheckerDispatcher { contract_address };

                let nonce_before_mock = dispatcher.get_nonce();
                let account_address_before_mock = dispatcher.get_account_contract_address();
                let version_before_mock = dispatcher.get_version();
                let max_fee_before_mock = dispatcher.get_max_fee();
                let chain_id_before_mock = dispatcher.get_chain_id();
                let signature_before_mock = dispatcher.get_signature();
                let tx_hash_before_mock = dispatcher.get_tx_hash();

                let mut tx_info_mock = TxInfoMockTrait::default();
                tx_info_mock.transaction_hash = Option::Some(421);
                start_spoof(contract_address, tx_info_mock);

                let transaction_hash = dispatcher.get_tx_hash();
                assert(transaction_hash == 421, 'Invalid tx hash');

                let nonce = dispatcher.get_nonce();
                assert(nonce == nonce_before_mock, 'Invalid nonce');
                let account_contract_address = dispatcher.get_account_contract_address();
                assert(
                    account_contract_address == account_address_before_mock,
                    'Invalid account address'
                );
                let version = dispatcher.get_version();
                assert(version == version_before_mock, 'Invalid version');
                let max_fee = dispatcher.get_max_fee();
                assert(max_fee == max_fee_before_mock, 'Invalid max_fee');
                let chain_id = dispatcher.get_chain_id();
                assert(chain_id == chain_id_before_mock, 'Invalid chain_id');
                let signature = dispatcher.get_signature();
                assert(signature == signature_before_mock, 'Invalid signature');

                stop_spoof(contract_address);

                let transaction_hash = dispatcher.get_tx_hash();
                assert(transaction_hash == tx_hash_before_mock, 'Invalid tx hash');
            }
        "#
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
            r#"
            use result::ResultTrait;
            use option::OptionTrait;
            use starknet::info::TxInfo;
            use serde::Serde;
            use traits::Into;
            use starknet::ContractAddress;
            use starknet::ContractAddressIntoFelt252;
            use starknet::Felt252TryIntoContractAddress;
            use array::SpanTrait;
            use snforge_std::{ declare, ContractClassTrait, start_spoof, TxInfoMock, TxInfoMockTrait };

            #[starknet::interface]
            trait ISpoofChecker<TContractState> {
                fn get_tx_hash(ref self: TContractState) -> felt252;
                fn get_nonce(ref self: TContractState) -> felt252;
                fn get_account_contract_address(ref self: TContractState) -> ContractAddress;
                fn get_signature(ref self: TContractState) -> Span<felt252>;
                fn get_version(ref self: TContractState) -> felt252;
                fn get_max_fee(ref self: TContractState) -> u128;
                fn get_chain_id(ref self: TContractState) -> felt252;
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

                start_spoof(contract_address, tx_info_mock);

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
            }
        "#
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
            r#"
            use result::ResultTrait;
            use box::BoxTrait;
            use starknet::info::TxInfo;
            use serde::Serde;
            use starknet::ContractAddress;
            use array::SpanTrait;
            use snforge_std::{ declare, ContractClassTrait, start_spoof, TxInfoMock, TxInfoMockTrait };

            #[starknet::interface]
            trait ISpoofChecker<TContractState> {
                fn get_tx_hash(ref self: TContractState) -> felt252;
                fn get_nonce(ref self: TContractState) -> felt252;
                fn get_account_contract_address(ref self: TContractState) -> ContractAddress;
                fn get_signature(ref self: TContractState) -> Span<felt252>;
                fn get_version(ref self: TContractState) -> felt252;
                fn get_max_fee(ref self: TContractState) -> u128;
                fn get_chain_id(ref self: TContractState) -> felt252;
            }

            #[test]
            fn start_spoof_cancel_mock_by_setting_attribute_to_none() {
                let contract = declare('SpoofChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpoofCheckerDispatcher { contract_address };

                let nonce_before_mock = dispatcher.get_nonce();
                let account_address_before_mock = dispatcher.get_account_contract_address();
                let version_before_mock = dispatcher.get_version();
                let max_fee_before_mock = dispatcher.get_max_fee();
                let chain_id_before_mock = dispatcher.get_chain_id();
                let signature_before_mock = dispatcher.get_signature();
                let tx_hash_before_mock = dispatcher.get_tx_hash();

                let mut tx_info_mock = TxInfoMockTrait::default();
                tx_info_mock.transaction_hash = Option::Some(421);
                start_spoof(contract_address, tx_info_mock);

                let transaction_hash = dispatcher.get_tx_hash();
                assert(transaction_hash == 421, 'Invalid tx hash');

                start_spoof(contract_address, TxInfoMockTrait::default());

                let transaction_hash = dispatcher.get_tx_hash();
                assert(transaction_hash == tx_hash_before_mock, 'Invalid tx hash');

                let nonce = dispatcher.get_nonce();
                assert(nonce == nonce_before_mock, 'Invalid nonce');
                let account_contract_address = dispatcher.get_account_contract_address();
                assert(
                    account_contract_address == account_address_before_mock,
                    'Invalid account address'
                );
                let version = dispatcher.get_version();
                assert(version == version_before_mock, 'Invalid version');
                let max_fee = dispatcher.get_max_fee();
                assert(max_fee == max_fee_before_mock, 'Invalid max_fee');
                let chain_id = dispatcher.get_chain_id();
                assert(chain_id == chain_id_before_mock, 'Invalid chain_id');
                let signature = dispatcher.get_signature();
                assert(signature == signature_before_mock, 'Invalid signature');
            }
        "#
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
            r#"
            use result::ResultTrait;
            use option::OptionTrait;
            use starknet::info::TxInfo;
            use serde::Serde;
            use traits::Into;
            use starknet::ContractAddress;
            use starknet::ContractAddressIntoFelt252;
            use array::SpanTrait;
            use snforge_std::{ declare, ContractClassTrait, start_spoof, TxInfoMock, TxInfoMockTrait };

            #[starknet::interface]
            trait ISpoofChecker<TContractState> {
                fn get_tx_hash(ref self: TContractState) -> felt252;
                fn get_nonce(ref self: TContractState) -> felt252;
                fn get_account_contract_address(ref self: TContractState) -> ContractAddress;
                fn get_signature(ref self: TContractState) -> Span<felt252>;
                fn get_version(ref self: TContractState) -> felt252;
                fn get_max_fee(ref self: TContractState) -> u128;
                fn get_chain_id(ref self: TContractState) -> felt252;
            }

            #[test]
            fn start_spoof_no_attributes_mocked() {
                let contract = declare('SpoofChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpoofCheckerDispatcher { contract_address };

                let nonce_before_mock = dispatcher.get_nonce();
                let tx_hash_before_mock = dispatcher.get_tx_hash();
                let account_address_before_mock = dispatcher.get_account_contract_address();
                let version_before_mock = dispatcher.get_version();
                let max_fee_before_mock = dispatcher.get_max_fee();
                let chain_id_before_mock = dispatcher.get_chain_id();
                let signature_before_mock = dispatcher.get_signature();

                let mut tx_info_mock = TxInfoMockTrait::default();
                start_spoof(contract_address, tx_info_mock);

                let transaction_hash = dispatcher.get_tx_hash();
                assert(transaction_hash == tx_hash_before_mock, 'Invalid tx hash');
                let nonce = dispatcher.get_nonce();
                assert(nonce == nonce_before_mock, 'Invalid nonce');
                let account_contract_address = dispatcher.get_account_contract_address();
                assert(
                    account_contract_address == account_address_before_mock,
                    'Invalid account address'
                );
                let version = dispatcher.get_version();
                assert(version == version_before_mock, 'Invalid version');
                let max_fee = dispatcher.get_max_fee();
                assert(max_fee == max_fee_before_mock, 'Invalid max_fee');
                let chain_id = dispatcher.get_chain_id();
                assert(chain_id == chain_id_before_mock, 'Invalid chain_id');
                let signature = dispatcher.get_signature();
                assert(signature.len() == signature_before_mock.len(), 'Invalid signature');
            }
        "#
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
