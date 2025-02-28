use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

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

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
#[ignore] // TODO(#2765)
#[expect(clippy::too_many_lines)]
fn start_cheat_execution_info_all_attributes_mocked() {
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
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, cheat_execution_info, ExecutionInfoMock, Operation, CheatArguments, CheatSpan};
            use starknet::info::v2::ResourceBounds;

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
            }

            #[test]
            fn start_cheat_execution_info_all_attributes_mocked() {
                let contract = declare("CheatTxInfoChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatTxInfoCheckerDispatcher { contract_address };

                let mut execution_info_mock: ExecutionInfoMock = Default::default();

                execution_info_mock.tx_info.nonce = Operation::Start(CheatArguments {
                    value: 411,
                    target: contract_address,
                    span: CheatSpan::Indefinite
                });
                execution_info_mock.tx_info.account_contract_address = Operation::Start(CheatArguments {
                    value: 422.try_into().unwrap(),
                    target: contract_address,
                    span: CheatSpan::Indefinite
                });
                execution_info_mock.tx_info.version = Operation::Start(CheatArguments {
                    value: 433,
                    target: contract_address,
                    span: CheatSpan::Indefinite
                });
                execution_info_mock.tx_info.transaction_hash = Operation::Start(CheatArguments {
                    value: 444,
                    target: contract_address,
                    span: CheatSpan::Indefinite
                });
                execution_info_mock.tx_info.chain_id = Operation::Start(CheatArguments {
                    value: 455,
                    target: contract_address,
                    span: CheatSpan::Indefinite
                });
                execution_info_mock.tx_info.max_fee = Operation::Start(CheatArguments {
                    value: 466,
                    target: contract_address,
                    span: CheatSpan::Indefinite
                });
                execution_info_mock.tx_info.signature = Operation::Start(CheatArguments {
                    value: array![477, 478].span(),
                    target: contract_address,
                    span: CheatSpan::Indefinite
                });
                execution_info_mock.tx_info.resource_bounds = Operation::Start(CheatArguments {
                    value: array![ResourceBounds { resource: 55, max_amount: 66, max_price_per_unit: 77 }, ResourceBounds { resource: 111, max_amount: 222, max_price_per_unit: 333 }].span(),
                    target: contract_address,
                    span: CheatSpan::Indefinite
                });
                execution_info_mock.tx_info.tip = Operation::Start(CheatArguments {
                    value: 123,
                    target: contract_address,
                    span: CheatSpan::Indefinite
                });
                execution_info_mock.tx_info.paymaster_data = Operation::Start(CheatArguments {
                    value: array![22, 33, 44].span(),
                    target: contract_address,
                    span: CheatSpan::Indefinite
                });
                execution_info_mock.tx_info.nonce_data_availability_mode = Operation::Start(CheatArguments {
                    value: 99,
                    target: contract_address,
                    span: CheatSpan::Indefinite
                });
                execution_info_mock.tx_info.fee_data_availability_mode = Operation::Start(CheatArguments {
                    value: 88,
                    target: contract_address,
                    span: CheatSpan::Indefinite
                });
                execution_info_mock.tx_info.account_deployment_data = Operation::Start(CheatArguments {
                    value: array![111, 222].span(),
                    target: contract_address,
                    span: CheatSpan::Indefinite
                });

                cheat_execution_info(execution_info_mock);

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
        "#
        ),
        Contract::from_code_path(
            "CheatTxInfoChecker".to_string(),
            Path::new("tests/data/contracts/cheat_tx_info_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

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
            use starknet::info::v2::ResourceBounds;

            #[starknet::interface]
            trait ICheatTxInfoChecker<TContractState> {
                fn get_tx_info(ref self: TContractState) -> starknet::info::v2::TxInfo;
            }

            #[test]
            fn start_cheat_transaction_hash_cancel_mock_by_setting_attribute_to_none() {
                let contract = declare("CheatTxInfoChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatTxInfoCheckerDispatcher { contract_address };

                let tx_info_before_mock = dispatcher.get_tx_info();

                start_cheat_transaction_hash(contract_address, 421);

                let mut expected_tx_info = tx_info_before_mock;
                expected_tx_info.transaction_hash = 421;

                assert_tx_info(dispatcher.get_tx_info(), expected_tx_info);

                stop_cheat_transaction_hash(contract_address);

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
        "#
        ),
        Contract::from_code_path(
            "CheatTxInfoChecker".to_string(),
            Path::new("tests/data/contracts/cheat_tx_info_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

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

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
#[ignore] // TODO(#2765)
#[expect(clippy::too_many_lines)]
fn start_cheat_execution_info_all() {
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
            use snforge_std::{ declare, ContractClassTrait, DeclareResultTrait, cheat_execution_info, ExecutionInfoMock, Operation, CheatArguments, CheatSpan };
            use starknet::info::v2::ResourceBounds;

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
            }

            #[test]
            fn start_cheat_execution_info_all_one_param() {
                let contract = declare("CheatTxInfoChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatTxInfoCheckerDispatcher { contract_address };

                let mut execution_info_mock: ExecutionInfoMock = Default::default();
                execution_info_mock.tx_info.transaction_hash = Operation::StartGlobal(421);
                cheat_execution_info(execution_info_mock);

                let transaction_hash = dispatcher.get_tx_hash();
                assert(transaction_hash == 421, 'Invalid tx hash');
            }

            #[test]
            fn start_cheat_execution_info_all_multiple_params() {
                let contract = declare("CheatTxInfoChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ICheatTxInfoCheckerDispatcher { contract_address };

                let mut execution_info_mock: ExecutionInfoMock = Default::default();

                execution_info_mock.tx_info.nonce = Operation::StartGlobal(411);
                execution_info_mock.tx_info.account_contract_address = Operation::StartGlobal(422.try_into().unwrap());
                execution_info_mock.tx_info.version = Operation::StartGlobal(433);
                execution_info_mock.tx_info.transaction_hash = Operation::StartGlobal(444);
                execution_info_mock.tx_info.chain_id = Operation::StartGlobal(455);
                execution_info_mock.tx_info.max_fee = Operation::StartGlobal(466_u128);
                execution_info_mock.tx_info.signature = Operation::StartGlobal(array![477, 478].span());
                execution_info_mock.tx_info.resource_bounds = Operation::StartGlobal(array![ResourceBounds { resource: 55, max_amount: 66, max_price_per_unit: 77 }, ResourceBounds { resource: 111, max_amount: 222, max_price_per_unit: 333 }].span());
                execution_info_mock.tx_info.tip = Operation::StartGlobal(123);
                execution_info_mock.tx_info.paymaster_data = Operation::StartGlobal(array![22, 33, 44].span());
                execution_info_mock.tx_info.nonce_data_availability_mode = Operation::StartGlobal(99);
                execution_info_mock.tx_info.fee_data_availability_mode = Operation::StartGlobal(88);
                execution_info_mock.tx_info.account_deployment_data = Operation::StartGlobal(array![111, 222].span());

                cheat_execution_info(execution_info_mock);

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
        "#
        ),
        Contract::from_code_path(
            "CheatTxInfoChecker".to_string(),
            Path::new("tests/data/contracts/cheat_tx_info_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

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

    let result = run_test_case(&test);

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
            use snforge_std::{ test_address, declare, ContractClassTrait, DeclareResultTrait, cheat_transaction_hash, stop_cheat_transaction_hash, CheatSpan};
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

    let result = run_test_case(&test);

    assert_passed(&result);
}
