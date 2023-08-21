use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::runner::Contract;
use crate::{assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;
use std::path::Path;

#[test]
fn start_spoof_single_attribute() {
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

    let result = run(
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &Default::default(),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}
