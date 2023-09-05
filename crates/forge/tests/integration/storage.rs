use crate::integration::common::running_tests::run_test_case;
use crate::{assert_passed, test_case};
use indoc::indoc;

#[test]
fn test_storage_access_from_tests() {
    let test = test_case!(indoc!(
        r#"
        
        #[starknet::contract]
        mod Contract {
            #[storage]
            struct Storage {
                balance: felt252, 
            }
            
            #[generate_trait]
            impl InternalImpl of InternalTrait {
                fn internal_function(self: @ContractState) -> felt252 {
                    self.balance.read()
                }
            }
        }

        use ___PREFIX_FOR_PACKAGE___test_case::Contract::balance::InternalContractMemberStateTrait;

        #[test]
        fn test_internal() {
            let mut state = Contract::contract_state_for_testing();
            state.balance.write(10);
            
            let value = Contract::InternalImpl::internal_function(@state);
            assert(value == 10, 'Incorrect storage value');
        }
    "#
    ),
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}
