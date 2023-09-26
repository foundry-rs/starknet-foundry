use crate::integration::common::runner::Contract;
use crate::integration::common::running_tests::run_test_case;
use crate::{assert_passed, test_case};
use indoc::indoc;

#[test]
fn error_handling() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use snforge_std::{ declare, ContractClass, ContractClassTrait };
        use array::ArrayTrait;

        #[test]
        fn test_deploy_error_handling() {
            let contract = declare('PanickingConstructor');

            match contract.deploy(@ArrayTrait::new()) {
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

    let result = run_test_case(&test);

    assert_passed!(result);
}
