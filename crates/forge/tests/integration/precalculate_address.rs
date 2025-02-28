use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn precalculate_address() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use snforge_std::{ declare, ContractClass, ContractClassTrait, DeclareResultTrait };
        use array::ArrayTrait;
        use traits::Into;
        use traits::TryInto;
        use starknet::ContractAddressIntoFelt252;

        #[test]
        fn precalculate_address() {
            let mut calldata = ArrayTrait::new();

            let contract = declare("HelloStarknet").unwrap().contract_class();
            let contract_address_pre = contract.precalculate_address(@calldata);
            let (contract_address, _) = contract.deploy(@calldata).unwrap();
            let contract_address_pre2 = contract.precalculate_address(@calldata);
            let (contract_address2, _) = contract.deploy(@calldata).unwrap();

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

    let result = run_test_case(&test);

    assert_passed(&result);
}
