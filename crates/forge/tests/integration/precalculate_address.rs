use indoc::indoc;
use std::path::Path;
use utils::runner::Contract;
use utils::running_tests::run_test_case;
use utils::{assert_passed, test_case};

#[test]
fn precalculate_address() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use snforge_std::{ declare, ContractClass, ContractClassTrait };
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

    let result = run_test_case(&test);

    assert_passed!(result);
}
