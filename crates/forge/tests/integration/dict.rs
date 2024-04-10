use indoc::indoc;
use std::path::Path;
use test_utils::runner::{assert_passed, Contract};
use test_utils::running_tests::run_test_case;

#[test]
fn using_dict() {
    let test = test_utils::test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use snforge_std::{ declare, ContractClass, ContractClassTrait };
        use array::ArrayTrait;
        
        #[starknet::interface]
        trait IDictUsingContract<TContractState> {
            fn get_unique(self: @TContractState) -> u8; 
            fn write_unique(self: @TContractState, values: Array<felt252>); 
        }
        
        #[test]
        fn using_dict() {
            let contract = declare("DictUsingContract").unwrap();
            let numbers = array![1, 2, 3, 3, 3, 3 ,3, 4, 4, 4, 4, 4, 5, 5, 5, 5];
            let mut inputs: Array<felt252> = array![];
            numbers.serialize(ref inputs);
            
            let (contract_address, _) = contract.deploy(@inputs).unwrap();
            let dispatcher = IDictUsingContractDispatcher { contract_address };
            
            let unq = dispatcher.get_unique();
            assert(unq == 5, 'wrong unique count');

            numbers.serialize(ref inputs);
            dispatcher.write_unique(array![1, 2, 3, 3, 3, 3 ,3, 4, 4, 4, 4, 4]);
            
            let unq = dispatcher.get_unique();
            assert(unq == 4, 'wrong unique count');
        }
        "#
        ),
        Contract::from_code_path(
            "DictUsingContract".to_string(),
            Path::new("tests/data/contracts/dict_using_contract.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
}
