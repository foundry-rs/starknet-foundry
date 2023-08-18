use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::runner::Contract;
use crate::{assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;
use std::path::Path;

#[test]
fn start_spoof_simple() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use box::BoxTrait;
            use starknet::info::TxInfo;
            use serde::Serde;
            use snforge_std::{ declare, ContractClassTrait, start_spoof, TxInfoMock, TxInfoMockTrait };

            #[starknet::interface]
            trait ISpoofChecker<TContractState> {
                fn get_tx_hash(ref self: TContractState) -> TxInfo;
            }

            #[test]
            fn start_spoof_simple() {
                let contract = declare('SpoofChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = ISpoofCheckerDispatcher { contract_address };

                let mut tx_info_mock = TxInfoMockTrait::default();
                tx_info_mock.transaction_hash = Option::Some(421);
                start_spoof(contract_address, tx_info_mock);

                let tx_info = dispatcher.get_tx_hash();

                assert(tx_info.transaction_hash == 421, 'Invalid tx hash');
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
