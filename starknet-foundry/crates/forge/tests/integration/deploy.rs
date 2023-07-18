use crate::common::corelib::{corelib, predeployed_contracts};
use crate::common::runner::Contract;
use crate::{assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;

#[test]
fn error_handling() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use cheatcodes::RevertedTransactionTrait;
        use cheatcodes::PreparedContract;
        use array::ArrayTrait;
        
        #[test]
        fn test_deploy_error_handling() {
            let class_hash = declare('PanickingConstructor').expect('Could not declare');
            let prepared_contract = PreparedContract {
                class_hash: class_hash,
                constructor_calldata: @ArrayTrait::new()
            };
        
            match deploy(prepared_contract) {
                Result::Ok(_) => panic_with_felt252('Should have panicked'),
                Result::Err(x) => {
                    assert(*x.panic_data.at(0_usize) == 'PANIK', *x.panic_data.at(0_usize));
                    assert(*x.panic_data.at(1_usize) == 'DEJTA', *x.panic_data.at(1_usize));
                }
            }
        }
    "#
        ),
        Contract::new(
            "PanickingConstructor",
            indoc!(
                r#"
                #[starknet::contract]
                mod PanickingConstructor {
                    use array::ArrayTrait;
                
                    #[storage]
                    struct Storage {}
                
                    #[constructor]
                    fn constructor(ref self: ContractState) {
                        let mut panic_data = ArrayTrait::new();
                        panic_data.append('PANIK');
                        panic_data.append('DEJTA');
                        panic(panic_data);
                    }
                }
    "#
            )
        )
    );

    let result = run(
        &test.path().unwrap(),
        &Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}
