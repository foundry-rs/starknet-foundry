use crate::utils::runner::{Contract, assert_passed};
use crate::utils::running_tests::run_test_case;
use crate::utils::test_case;
use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use std::path::Path;

#[test]
fn start_and_stop_cheat_transaction_hash_single_attribute() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use box::BoxTrait;
            use starknet::info::TxInfo;
            use serde::Serde;
            use starknet::ContractAddress;
            use array::SpanTrait;
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_cheat_transaction_hash, start_cheat_transaction_hash_global, stop_cheat_transaction_hash };
            use starknet::info::v2::ResourceBounds;

            #[starknet::interface]
            trait ICheatTxInfoChecker<TContractState> {
                fn get_tx_info(ref self: TContractState) -> starknet::info::v2::TxInfo;
            }

            #[test]
            fn start_cheat_transaction_hash_single_attribute() {
                let contract = declare("CheatTxInfoChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatTxInfoCheckerDispatcher { contract_address };

                let tx_info_before = dispatcher.get_tx_info();

                start_cheat_transaction_hash(contract_address, 421);

                let mut expected_tx_info = tx_info_before;
                expected_tx_info.transaction_hash = 421;

                assert_tx_info(dispatcher.get_tx_info(), expected_tx_info);

                stop_cheat_transaction_hash(contract_address);

                assert_tx_info(dispatcher.get_tx_info(), tx_info_before);
            }

            #[test]
            fn test_cheat_transaction_hash_all_stop_one() {
                let contract = declare("CheatTxInfoChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatTxInfoCheckerDispatcher { contract_address };

                let tx_info_before = dispatcher.get_tx_info();

                start_cheat_transaction_hash_global(421);

                let mut expected_tx_info = tx_info_before;
                expected_tx_info.transaction_hash = 421;

                assert_tx_info(dispatcher.get_tx_info(), expected_tx_info);

                stop_cheat_transaction_hash(contract_address);

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
        "#
        ),
        Contract::from_code_path(
            "CheatTxInfoChecker".to_string(),
            Path::new("tests/data/contracts/cheat_tx_info_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

// TODO(#2765): Test below was intended to test private `ExecutionInfoMock` and `cheat_execution_info`
#[test]
#[expect(clippy::too_many_lines)]
fn start_cheat_execution_info_all_per_contract() {
    let test = test_case!(
        indoc!(
            r#"
            use array::SpanTrait;
            use option::OptionTrait;
            use result::ResultTrait;
            use serde::Serde;
            use snforge_std::{
                ContractClassTrait, DeclareResultTrait, declare, start_cheat_account_contract_address,
                start_cheat_account_deployment_data, start_cheat_chain_id,
                start_cheat_fee_data_availability_mode, start_cheat_max_fee, start_cheat_nonce,
                start_cheat_nonce_data_availability_mode, start_cheat_paymaster_data, start_cheat_proof_facts,
                start_cheat_resource_bounds, start_cheat_signature, start_cheat_tip,
                start_cheat_transaction_hash, start_cheat_transaction_version,
            };
            use starknet::info::TxInfo;
            use starknet::info::v3::ResourceBounds;
            use starknet::{ContractAddress, ContractAddressIntoFelt252, Felt252TryIntoContractAddress};
            use traits::Into;

            #[starknet::interface]
            trait ICheatTxInfoChecker<TContractState> {
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
                fn get_proof_facts(self: @TContractState) -> Span<felt252>;
            }

            #[test]
            fn start_cheat_execution_info_all_attributes_mocked() {
                let contract = declare("CheatTxInfoChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatTxInfoCheckerDispatcher { contract_address };

                start_cheat_nonce(contract_address, 411);
                start_cheat_account_contract_address(contract_address, 422.try_into().unwrap());
                start_cheat_transaction_version(contract_address, 433);
                start_cheat_transaction_hash(contract_address, 444);
                start_cheat_chain_id(contract_address, 455);
                start_cheat_max_fee(contract_address, 466);
                start_cheat_signature(contract_address, array![477, 478].span());
                start_cheat_resource_bounds(
                    contract_address,
                    array![
                        ResourceBounds { resource: 55, max_amount: 66, max_price_per_unit: 77 },
                        ResourceBounds { resource: 111, max_amount: 222, max_price_per_unit: 333 },
                    ]
                        .span(),
                );
                start_cheat_tip(contract_address, 123);
                start_cheat_paymaster_data(contract_address, array![22, 33, 44].span());
                start_cheat_nonce_data_availability_mode(contract_address, 99);
                start_cheat_fee_data_availability_mode(contract_address, 88);
                start_cheat_account_deployment_data(contract_address, array![111, 222].span());
                start_cheat_proof_facts(contract_address, array![19, 38, 57, 76].span());

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

                let proof_facts = dispatcher.get_proof_facts();
                assert(proof_facts == array![19, 38, 57, 76].span(), 'Invalid proof facts');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatTxInfoChecker".to_string(),
            Path::new("tests/data/contracts/cheat_tx_info_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn start_cheat_transaction_hash_cancel_mock_by_setting_attribute_to_none() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use box::BoxTrait;
            use starknet::info::TxInfo;
            use serde::Serde;
            use starknet::ContractAddress;
            use array::SpanTrait;
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_cheat_transaction_hash, stop_cheat_transaction_hash, CheatSpan };
            use starknet::info::v3::ResourceBounds;

            #[starknet::interface]
            trait ICheatTxInfoChecker<TContractState> {
                fn get_tx_info_v3(ref self: TContractState) -> starknet::info::v3::TxInfo;
            }

            #[test]
            fn start_cheat_transaction_hash_cancel_mock_by_setting_attribute_to_none() {
                let contract = declare("CheatTxInfoChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatTxInfoCheckerDispatcher { contract_address };

                let tx_info_before_mock = dispatcher.get_tx_info_v3();

                start_cheat_transaction_hash(contract_address, 421);

                let mut expected_tx_info = tx_info_before_mock;
                expected_tx_info.transaction_hash = 421;

                assert_tx_info(dispatcher.get_tx_info_v3(), expected_tx_info);

                stop_cheat_transaction_hash(contract_address);

                assert_tx_info(dispatcher.get_tx_info_v3(), tx_info_before_mock);
            }

            fn assert_tx_info(tx_info: starknet::info::v3::TxInfo, expected_tx_info: starknet::info::v3::TxInfo) {
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
                assert(tx_info.proof_facts == expected_tx_info.proof_facts, 'Invalid proof_facts');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatTxInfoChecker".to_string(),
            Path::new("tests/data/contracts/cheat_tx_info_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn start_cheat_transaction_hash_multiple() {
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
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_cheat_transaction_hash, CheatSpan};

            #[starknet::interface]
            trait ICheatTxInfoChecker<TContractState> {
                fn get_tx_hash(ref self: TContractState) -> felt252;
            }

            #[test]
            fn start_cheat_transaction_hash_multiple() {
                let contract = declare("CheatTxInfoChecker").unwrap().contract_class();

                let (contract_address_1, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher_1 = ICheatTxInfoCheckerDispatcher { contract_address: contract_address_1 };

                let (contract_address_2, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher_2 = ICheatTxInfoCheckerDispatcher { contract_address: contract_address_2 };

                start_cheat_transaction_hash(contract_address_1, 421);
                start_cheat_transaction_hash(contract_address_2, 421);

                let transaction_hash = dispatcher_1.get_tx_hash();
                assert(transaction_hash == 421, 'Invalid tx hash');

                let transaction_hash = dispatcher_2.get_tx_hash();
                assert(transaction_hash == 421, 'Invalid tx hash');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatTxInfoChecker".to_string(),
            Path::new("tests/data/contracts/cheat_tx_info_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

// TODO(#2765): Test below was intended to test private `ExecutionInfoMock` and `cheat_execution_info`
#[test]
#[expect(clippy::too_many_lines)]
fn start_cheat_execution_info_all_global() {
    let test = test_case!(
        indoc!(
            r#"
            use array::SpanTrait;
            use option::OptionTrait;
            use result::ResultTrait;
            use serde::Serde;
            use snforge_std::{
                ContractClassTrait, DeclareResultTrait, declare, start_cheat_account_contract_address_global,
                start_cheat_account_deployment_data_global, start_cheat_chain_id_global,
                start_cheat_fee_data_availability_mode_global, start_cheat_max_fee_global,
                start_cheat_nonce_data_availability_mode_global, start_cheat_nonce_global,
                start_cheat_paymaster_data_global, start_cheat_proof_facts_global,
                start_cheat_resource_bounds_global, start_cheat_signature_global, start_cheat_tip_global,
                start_cheat_transaction_hash_global, start_cheat_transaction_version_global,
            };
            use starknet::info::TxInfo;
            use starknet::info::v2::ResourceBounds;
            use starknet::{ContractAddress, ContractAddressIntoFelt252};
            use traits::Into;

            #[starknet::interface]
            trait ICheatTxInfoChecker<TContractState> {
                fn get_tx_hash(self: @TContractState) -> felt252;
                fn get_nonce(self: @TContractState) -> felt252;
                fn get_account_contract_address(self: @TContractState) -> ContractAddress;
                fn get_signature(self: @TContractState) -> Span<felt252>;
                fn get_version(self: @TContractState) -> felt252;
                fn get_max_fee(self: @TContractState) -> u128;
                fn get_chain_id(self: @TContractState) -> felt252;
                fn get_resource_bounds(self: @TContractState) -> Span<ResourceBounds>;
                fn get_tip(self: @TContractState) -> u128;
                fn get_paymaster_data(self: @TContractState) -> Span<felt252>;
                fn get_nonce_data_availability_mode(self: @TContractState) -> u32;
                fn get_fee_data_availability_mode(self: @TContractState) -> u32;
                fn get_account_deployment_data(self: @TContractState) -> Span<felt252>;
                fn get_proof_facts(self: @TContractState) -> Span<felt252>;
                fn get_tx_info(self: @TContractState) -> starknet::info::v2::TxInfo;
                fn get_tx_info_v3(self: @TContractState) -> starknet::info::v3::TxInfo;
            }

            #[test]
            fn start_cheat_execution_info_all_one_param() {
                let contract = declare("CheatTxInfoChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatTxInfoCheckerDispatcher { contract_address };

                start_cheat_transaction_hash_global(421);

                let transaction_hash = dispatcher.get_tx_hash();
                assert(transaction_hash == 421, 'Invalid tx hash');
            }

            #[test]
            fn start_cheat_execution_info_all_multiple_params() {
                let contract = declare("CheatTxInfoChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatTxInfoCheckerDispatcher { contract_address };

                start_cheat_nonce_global(411);
                start_cheat_account_contract_address_global(422.try_into().unwrap());
                start_cheat_transaction_version_global(433);
                start_cheat_transaction_hash_global(444);
                start_cheat_chain_id_global(455);
                start_cheat_max_fee_global(466_u128);
                start_cheat_signature_global(array![477, 478].span());
                start_cheat_resource_bounds_global(
                    array![
                        ResourceBounds { resource: 55, max_amount: 66, max_price_per_unit: 77 },
                        ResourceBounds { resource: 111, max_amount: 222, max_price_per_unit: 333 },
                    ]
                        .span(),
                );
                start_cheat_tip_global(123);
                start_cheat_paymaster_data_global(array![22, 33, 44].span());
                start_cheat_nonce_data_availability_mode_global(99);
                start_cheat_fee_data_availability_mode_global(88);
                start_cheat_account_deployment_data_global(array![111, 222].span());
                start_cheat_proof_facts_global(array![999, 1001].span());

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

                let proof_facts = dispatcher.get_proof_facts();
                assert(proof_facts == array![999, 1001].span(), 'Invalid proof facts');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatTxInfoChecker".to_string(),
            Path::new("tests/data/contracts/cheat_tx_info_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn start_cheat_transaction_hash_complex() {
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
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, start_cheat_transaction_hash, start_cheat_transaction_hash_global, CheatSpan };

            #[starknet::interface]
            trait ICheatTxInfoChecker<TContractState> {
                fn get_tx_hash(ref self: TContractState) -> felt252;
            }

            #[test]
            fn start_cheat_transaction_hash_complex() {
                let contract = declare("CheatTxInfoChecker").unwrap().contract_class();
                let (contract_address_1, _) = contract.deploy(@array![]).unwrap();
                let (contract_address_2, _) = contract.deploy(@array![]).unwrap();

                let dispatcher_1 = ICheatTxInfoCheckerDispatcher { contract_address: contract_address_1 };
                let dispatcher_2 = ICheatTxInfoCheckerDispatcher { contract_address: contract_address_2 };

                start_cheat_transaction_hash_global(421);

                let transaction_hash_1 = dispatcher_1.get_tx_hash();
                let transaction_hash_2 = dispatcher_2.get_tx_hash();
                assert(transaction_hash_1 == 421, 'Invalid tx hash');
                assert(transaction_hash_2 == 421, 'Invalid tx hash');

                start_cheat_transaction_hash(contract_address_2, 621);

                let transaction_hash_1 = dispatcher_1.get_tx_hash();
                let transaction_hash_2 = dispatcher_2.get_tx_hash();
                assert(transaction_hash_1 == 421, 'Invalid tx hash');
                assert(transaction_hash_2 == 621, 'Invalid tx hash');

                start_cheat_transaction_hash(contract_address_1, 821);
                start_cheat_transaction_hash(contract_address_2, 821);

                let transaction_hash_1 = dispatcher_1.get_tx_hash();
                let transaction_hash_2 = dispatcher_2.get_tx_hash();
                assert(transaction_hash_1 == 821, 'Invalid tx hash');
                assert(transaction_hash_2 == 821, 'Invalid tx hash');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatTxInfoChecker".to_string(),
            Path::new("tests/data/contracts/cheat_tx_info_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn cheat_transaction_hash_with_span() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use box::BoxTrait;
            use starknet::info::TxInfo;
            use serde::Serde;
            use starknet::ContractAddress;
            use array::SpanTrait;
            use snforge_std::{ test_address, declare, ContractClassTrait, DeclareResultTrait, cheat_transaction_hash, stop_cheat_transaction_hash, CheatSpan };
            use starknet::info::v2::ResourceBounds;

            #[starknet::interface]
            trait ICheatTxInfoChecker<TContractState> {
                fn get_tx_info(ref self: TContractState) -> starknet::info::v2::TxInfo;
            }

            fn deploy_cheat_transaction_hash_checker() -> ICheatTxInfoCheckerDispatcher {
                let (contract_address, _) = declare("CheatTxInfoChecker").unwrap().contract_class().deploy(@ArrayTrait::new()).unwrap();
                ICheatTxInfoCheckerDispatcher { contract_address }
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

            #[test]
            fn test_cheat_transaction_hash_once() {
                let dispatcher = deploy_cheat_transaction_hash_checker();

                let tx_info_before = dispatcher.get_tx_info();

                cheat_transaction_hash(dispatcher.contract_address, 421, CheatSpan::TargetCalls(1));

                let mut expected_tx_info = tx_info_before;
                expected_tx_info.transaction_hash = 421;

                assert_tx_info(dispatcher.get_tx_info(), expected_tx_info);
                assert_tx_info(dispatcher.get_tx_info(), tx_info_before);
            }

            #[test]
            fn test_cheat_transaction_hash_twice() {
                let dispatcher = deploy_cheat_transaction_hash_checker();

                let tx_info_before = dispatcher.get_tx_info();

                cheat_transaction_hash(dispatcher.contract_address, 421, CheatSpan::TargetCalls(2));

                let mut expected_tx_info = tx_info_before;
                expected_tx_info.transaction_hash = 421;

                assert_tx_info(dispatcher.get_tx_info(), expected_tx_info);
                assert_tx_info(dispatcher.get_tx_info(), expected_tx_info);
                assert_tx_info(dispatcher.get_tx_info(), tx_info_before);
            }

            #[test]
            fn test_cheat_transaction_hash_test_address() {
                let tx_info_before = starknet::get_tx_info().unbox();

                cheat_transaction_hash(test_address(), 421,CheatSpan::TargetCalls(1) );

                let mut expected_tx_info = tx_info_before;
                expected_tx_info.transaction_hash = 421;

                let tx_info = starknet::get_tx_info().unbox();
                assert_tx_info(tx_info, expected_tx_info);

                stop_cheat_transaction_hash(test_address());

                let tx_info = starknet::get_tx_info().unbox();
                assert_tx_info(tx_info, tx_info_before);
            }
        "#
        ),
        Contract::from_code_path(
            "CheatTxInfoChecker".to_string(),
            Path::new("tests/data/contracts/cheat_tx_info_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

/// Verify that `block_number`, `block_timestamp`, `sequencer_address` and `transaction_hash` cheats
/// applied to `test_address()` are visible inside library calls made directly from test code.
#[test]
#[allow(clippy::too_many_lines)]
fn cheat_execution_info_direct_library_call_from_test() {
    let test = test_case!(
        indoc!(
            r#"
            use starknet::ContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, test_address,
                start_cheat_block_number, stop_cheat_block_number,
                start_cheat_block_timestamp, stop_cheat_block_timestamp,
                start_cheat_sequencer_address, stop_cheat_sequencer_address,
                start_cheat_transaction_hash, stop_cheat_transaction_hash,
            };

            #[starknet::interface]
            trait ICheatBlockNumberChecker<TContractState> {
                fn get_block_number(ref self: TContractState) -> u64;
            }

            #[starknet::interface]
            trait ICheatBlockTimestampChecker<TContractState> {
                fn get_block_timestamp(ref self: TContractState) -> u64;
            }

            #[starknet::interface]
            trait ICheatSequencerAddressChecker<TContractState> {
                fn get_sequencer_address(ref self: TContractState) -> ContractAddress;
            }

            #[starknet::interface]
            trait ICheatTxInfoChecker<TContractState> {
                fn get_tx_hash(self: @TContractState) -> felt252;
            }

            #[test]
            fn test_cheat_block_number_direct_library_call_from_test() {
                let class_hash = *declare("CheatBlockNumberChecker").unwrap().contract_class().class_hash;
                let lib_dispatcher = ICheatBlockNumberCheckerLibraryDispatcher { class_hash };

                let original = lib_dispatcher.get_block_number();

                start_cheat_block_number(test_address(), 1234_u64);

                let cheated = lib_dispatcher.get_block_number();
                assert(cheated == 1234, 'Wrong block number');

                stop_cheat_block_number(test_address());

                let restored = lib_dispatcher.get_block_number();
                assert(restored == original, 'Block number not restored');
            }

            #[test]
            fn test_cheat_block_timestamp_direct_library_call_from_test() {
                let class_hash = *declare("CheatBlockTimestampChecker").unwrap().contract_class().class_hash;
                let lib_dispatcher = ICheatBlockTimestampCheckerLibraryDispatcher { class_hash };

                let original = lib_dispatcher.get_block_timestamp();

                start_cheat_block_timestamp(test_address(), 9999_u64);

                let cheated = lib_dispatcher.get_block_timestamp();
                assert(cheated == 9999, 'Wrong block timestamp');

                stop_cheat_block_timestamp(test_address());

                let restored = lib_dispatcher.get_block_timestamp();
                assert(restored == original, 'Block timestamp not restored');
            }

            #[test]
            fn test_cheat_sequencer_address_direct_library_call_from_test() {
                let class_hash = *declare("CheatSequencerAddressChecker").unwrap().contract_class().class_hash;
                let lib_dispatcher = ICheatSequencerAddressCheckerLibraryDispatcher { class_hash };

                let original = lib_dispatcher.get_sequencer_address();

                let target_sequencer: ContractAddress = 42.try_into().unwrap();
                start_cheat_sequencer_address(test_address(), target_sequencer);

                let cheated = lib_dispatcher.get_sequencer_address();
                assert(cheated == target_sequencer, 'Wrong sequencer address');

                stop_cheat_sequencer_address(test_address());

                let restored = lib_dispatcher.get_sequencer_address();
                assert(restored == original, 'Sequencer address not restored');
            }

            #[test]
            fn test_cheat_transaction_hash_direct_library_call_from_test() {
                let class_hash = *declare("CheatTxInfoChecker").unwrap().contract_class().class_hash;
                let lib_dispatcher = ICheatTxInfoCheckerLibraryDispatcher { class_hash };

                let original = lib_dispatcher.get_tx_hash();

                start_cheat_transaction_hash(test_address(), 0xdeadbeef);

                let cheated = lib_dispatcher.get_tx_hash();
                assert(cheated == 0xdeadbeef, 'Wrong tx hash');

                stop_cheat_transaction_hash(test_address());

                let restored = lib_dispatcher.get_tx_hash();
                assert(restored == original, 'Tx hash not restored');
            }
            "#
        ),
        Contract::from_code_path(
            "CheatBlockNumberChecker".to_string(),
            Path::new("tests/data/contracts/cheat_block_number_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "CheatBlockTimestampChecker".to_string(),
            Path::new("tests/data/contracts/cheat_block_timestamp_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "CheatSequencerAddressChecker".to_string(),
            Path::new("tests/data/contracts/cheat_sequencer_address_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "CheatTxInfoChecker".to_string(),
            Path::new("tests/data/contracts/cheat_tx_info_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_passed(&result);
}

/// Verify that cheats applied to a contract address are visible when that contract internally
/// makes library calls. Tests `block_number`, `block_timestamp`, `sequencer_address` and `tx_hash`.
#[test]
#[allow(clippy::too_many_lines)]
fn cheat_execution_info_library_call_from_contract() {
    let test = test_case!(
        indoc!(
            r#"
            use starknet::ContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, test_address,
                start_cheat_block_number, stop_cheat_block_number,
                start_cheat_block_timestamp, stop_cheat_block_timestamp,
                start_cheat_sequencer_address, stop_cheat_sequencer_address,
                start_cheat_transaction_hash, stop_cheat_transaction_hash,
            };

            #[starknet::interface]
            trait ICheatExecutionInfoLibraryCallChecker<TContractState> {
                fn get_block_number_via_library_call(
                    ref self: TContractState, class_hash: starknet::ClassHash,
                ) -> u64;
                fn get_block_timestamp_via_library_call(
                    ref self: TContractState, class_hash: starknet::ClassHash,
                ) -> u64;
                fn get_sequencer_address_via_library_call(
                    ref self: TContractState, class_hash: starknet::ClassHash,
                ) -> ContractAddress;
                fn get_tx_hash_via_library_call(
                    self: @TContractState, class_hash: starknet::ClassHash,
                ) -> felt252;
            }

            fn deploy_proxy() -> (ICheatExecutionInfoLibraryCallCheckerDispatcher, ContractAddress) {
                let proxy = declare("CheatExecutionInfoLibraryCallChecker").unwrap().contract_class();
                let (proxy_address, _) = proxy.deploy(@array![]).unwrap();
                (ICheatExecutionInfoLibraryCallCheckerDispatcher { contract_address: proxy_address }, proxy_address)
            }

            #[test]
            fn test_cheat_block_number_library_call_from_contract() {
                let block_number_class_hash = *declare("CheatBlockNumberChecker").unwrap().contract_class().class_hash;
                let (proxy, proxy_address) = deploy_proxy();

                let original = proxy.get_block_number_via_library_call(block_number_class_hash);

                start_cheat_block_number(proxy_address, 5678_u64);

                let cheated = proxy.get_block_number_via_library_call(block_number_class_hash);
                assert(cheated == 5678, 'Wrong block number');

                stop_cheat_block_number(proxy_address);

                let restored = proxy.get_block_number_via_library_call(block_number_class_hash);
                assert(restored == original, 'Block number not restored');
            }

            #[test]
            fn test_cheat_block_timestamp_library_call_from_contract() {
                let class_hash = *declare("CheatBlockTimestampChecker").unwrap().contract_class().class_hash;
                let (proxy, proxy_address) = deploy_proxy();

                let original = proxy.get_block_timestamp_via_library_call(class_hash);

                start_cheat_block_timestamp(proxy_address, 7777_u64);

                let cheated = proxy.get_block_timestamp_via_library_call(class_hash);
                assert(cheated == 7777, 'Wrong block timestamp');

                stop_cheat_block_timestamp(proxy_address);

                let restored = proxy.get_block_timestamp_via_library_call(class_hash);
                assert(restored == original, 'Block timestamp not restored');
            }

            #[test]
            fn test_cheat_sequencer_address_library_call_from_contract() {
                let class_hash = *declare("CheatSequencerAddressChecker").unwrap().contract_class().class_hash;
                let (proxy, proxy_address) = deploy_proxy();

                let original = proxy.get_sequencer_address_via_library_call(class_hash);

                let target_sequencer: ContractAddress = 99.try_into().unwrap();
                start_cheat_sequencer_address(proxy_address, target_sequencer);

                let cheated = proxy.get_sequencer_address_via_library_call(class_hash);
                assert(cheated == target_sequencer, 'Wrong sequencer address');

                stop_cheat_sequencer_address(proxy_address);

                let restored = proxy.get_sequencer_address_via_library_call(class_hash);
                assert(restored == original, 'Sequencer address not restored');
            }

            #[test]
            fn test_cheat_transaction_hash_library_call_from_contract() {
                let class_hash = *declare("CheatTxInfoChecker").unwrap().contract_class().class_hash;
                let (proxy, proxy_address) = deploy_proxy();

                let original = proxy.get_tx_hash_via_library_call(class_hash);

                start_cheat_transaction_hash(proxy_address, 0xcafebabe);

                let cheated = proxy.get_tx_hash_via_library_call(class_hash);
                assert(cheated == 0xcafebabe, 'Wrong tx hash');

                stop_cheat_transaction_hash(proxy_address);

                let restored = proxy.get_tx_hash_via_library_call(class_hash);
                assert(restored == original, 'Tx hash not restored');
            }
            "#
        ),
        Contract::from_code_path(
            "CheatBlockNumberChecker".to_string(),
            Path::new("tests/data/contracts/cheat_block_number_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "CheatBlockTimestampChecker".to_string(),
            Path::new("tests/data/contracts/cheat_block_timestamp_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "CheatSequencerAddressChecker".to_string(),
            Path::new("tests/data/contracts/cheat_sequencer_address_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "CheatTxInfoChecker".to_string(),
            Path::new("tests/data/contracts/cheat_tx_info_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "CheatExecutionInfoLibraryCallChecker".to_string(),
            Path::new("tests/data/contracts/cheat_tx_info_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_passed(&result);
}
