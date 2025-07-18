use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use test_utils::runner::{Contract, assert_passed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
#[cfg_attr(not(feature = "interact-with-state"), ignore)]
fn get_contract_address_in_interact_with_state() {
    let test = test_case!(
        indoc!(
            r#"
        use snforge_std::{
            ContractClassTrait, DeclareResultTrait, declare, interact_with_state, test_address,
        };
        use starknet::{ContractAddress, get_contract_address};

        #[starknet::interface]
        trait IEmpty<TContractState> {
            fn get_address(ref self: TContractState) -> ContractAddress;
        }

        #[test]
        fn test_contract_address_set_correctly() {
            let contract = declare("Empty").unwrap().contract_class();
            let (contract_address, _) = contract.deploy(@array![]).unwrap();
            let (other_empty_contract, _) = contract.deploy(@array![]).unwrap();
            let dispatcher = IEmptyDispatcher { contract_address };
            let other_dispatcher = IEmptyDispatcher { contract_address: other_empty_contract };

            let assert_eq_addresses = |a: ContractAddress, b: ContractAddress| {
                assert(a == b, 'Incorrect address');
            };

            assert_eq_addresses(dispatcher.get_address(), contract_address);
            assert_eq_addresses(get_contract_address(), test_address());

            interact_with_state(
                contract_address,
                || {
                    assert_eq_addresses(dispatcher.get_address(), contract_address);
                    assert_eq_addresses(get_contract_address(), contract_address);

                    // Make sure other contracts are not modified
                    assert_eq_addresses(other_dispatcher.get_address(), other_empty_contract);
                },
            );

            // Make sure `get_contract_address` was modified only for the `interact_with_state` execution
            assert_eq_addresses(dispatcher.get_address(), contract_address);
            assert_eq_addresses(get_contract_address(), test_address());
        }
            "#
        ),
        Contract::new(
            "Empty",
            indoc!(
                r"
            #[starknet::interface]
            trait IEmpty<TContractState> {
                fn get_address(ref self: TContractState) -> starknet::ContractAddress;
            }

            #[starknet::contract]
            mod Empty {
                #[storage]
                struct Storage {}

                #[abi(embed_v0)]
                impl EmptyImpl of super::IEmpty<ContractState> {
                    fn get_address(ref self: ContractState) -> starknet::ContractAddress {
                        starknet::get_contract_address()
                    }
                }
            }
            "
            )
        )
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}
