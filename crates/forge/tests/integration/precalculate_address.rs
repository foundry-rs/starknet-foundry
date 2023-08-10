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

        #[starknet::interface]
        trait IHelloStarknet<TContractState> {
            fn increase_balance(ref self: TContractState, amount: felt252);
            fn get_balance(self: @TContractState) -> felt252;
            fn do_a_panic(self: @TContractState);
            fn do_a_panic_with(self: @TContractState, panic_data: Array<felt252>);
        }
            
        #[test]
        fn get_contract_precalculate_address() {
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
