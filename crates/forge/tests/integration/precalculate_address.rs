use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::runner::Contract;
use crate::{assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;
use std::path::Path;

#[test]
fn test_get_contract_precalculate_address() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use cheatcodes::{ declare, ContractClass, ContractClassTrait };
        use array::ArrayTrait;
        use traits::Into;
        use traits::TryInto;
        use starknet::ContractAddressIntoFelt252;

            
        #[test]
        fn get_contract_precalculate_address() {
            let mut calldata = ArrayTrait::new();

            let contract = declare('HelloStarknet');
            let contract_address_pre = contract.precalculate_address(@calldata);
            let contract_address = contract.deploy(@calldata).unwrap();
            let contract_address_pre2 = contract.precalculate_address(@calldata);
            let contract_address2 = contract.deploy(@calldata).unwrap();

            assert(contract_address_pre == contract_address, 'must be eq');
            assert(contract_address_pre2 == contract_address2, 'must be eq');
            assert(contract_address != contract_address2, 'must be different');
        }
    "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet.cairo"),
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

#[test]
fn test_precalculate_address_with_calldata() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use cheatcodes::{ declare, ContractClass, ContractClassTrait };
        use array::ArrayTrait;
        use traits::Into;
        use traits::TryInto;
        use starknet::ContractAddressIntoFelt252;

        #[test]
        fn get_contract_precalculate_address() {
            let mut calldata = ArrayTrait::new();
            calldata.append(2);

            let mut calldata2 = ArrayTrait::new();
            calldata2.append(3);

            let contract = declare('ConstructorSimple');
            let addr1_pre = contract.precalculate_address(@calldata);
            let addr2_pre = contract.precalculate_address(@calldata2);
            let addr1 = contract.deploy(@calldata).unwrap();
            let addr2_after = contract.precalculate_address(@calldata2);
            let addr2 = contract.deploy(@calldata2).unwrap();

            assert(addr1_pre == addr1, 'must be eq');
            assert(addr1_pre != addr2_pre, 'must be different');
            assert(addr2_pre != addr2_after &&  addr2_after == addr2, '');
        }
    "#
        ),
        Contract::from_code_path(
            "ConstructorSimple".to_string(),
            Path::new("tests/data/contracts/constructor_simple.cairo"),
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
