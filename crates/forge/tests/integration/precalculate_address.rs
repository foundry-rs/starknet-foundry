use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::{assert_failed, assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;

#[test]
fn simple() {
    let test = test_case!(indoc!(
        r#"
        use result::ResultTrait;
        use cheatcodes::{ declare, ContractClass, ContractClassTrait };
        use array::ArrayTrait;
        use traits::Into;
        use traits::TryInto;
        use starknet::ContractAddressIntoFelt252;
            
        #[test]
        fn deploy_invalid_calldata() {
            let mut calldata = ArrayTrait::new();

            let contract = declare('HelloStarknet');
            let contract_address_pre = contract.precalculate_address(@calldata);
            let contract_address = contract.deploy(@calldata).unwrap();
            let contract_address_pre2 = contract.precalculate_address(@calldata);
            let contract_address2 = contract.deploy(@calldata).unwrap();

            assert(contract_address_pre.into() == contract_address, contract_address.into());
            assert(contract_address_pre2.into() == contract_address2, contract_address.into());
        }
    "#
    ));

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
