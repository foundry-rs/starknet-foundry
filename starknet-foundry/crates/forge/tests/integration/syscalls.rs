use crate::integration::common::corelib::{corelib, predeployed_contracts};
use crate::integration::common::runner::Contract;
use crate::{assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;

#[test]
fn library_call_syscall() {
    // function_selector calculated using starknet-py's `get_selector_from_name`
    // https://starknetpy.readthedocs.io/en/latest/guide/account_and_client.html#executing-transactions
    let test = test_case!(
        indoc!(
            r#"
        use array::ArrayTrait;
        use result::ResultTrait;
        use option::OptionTrait;
        use traits::TryInto;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        
        #[starknet::interface]
        trait ICaller<TContractState> {
            fn call_libfunc_syscall(
                self: @TContractState, class_hash: felt252, function_selector: felt252, calldata: Span<felt252>
            ) -> Span<felt252>;
        }
        
        #[test]
        fn test_increase_balance() {
            let caller_class_hash = declare('Caller').unwrap();
            let prepared = PreparedContract {
                class_hash: caller_class_hash, constructor_calldata: @ArrayTrait::new()
            };
            let contract_address = deploy(prepared).unwrap();
        
            let safe_dispatcher = ICallerSafeDispatcher {
                contract_address: contract_address.try_into().unwrap()
            };
        
            let executor_class_hash = declare('Executor').unwrap();
            let mut calldata = ArrayTrait::new();
            calldata.append(420);
        
            let result = safe_dispatcher
                .call_libfunc_syscall(
                    executor_class_hash,
                    513583194757451964557877661208561352178142916039616450970863240829819831951,
                    calldata.span()
                )
                .unwrap();
        
            assert(*result[0] == 422, 'Invalid balance');
        }
        "#
        ),
        Contract::new(
            "Caller",
            indoc!(
                r#"
                #[starknet::contract]
                mod Caller {
                    use starknet::ClassHash;
                    use starknet::class_hash_try_from_felt252;
                    use starknet::library_call_syscall;
                
                    use option::OptionTrait;
                    use result::ResultTrait;
                
                    #[storage]
                    struct Storage {}
                
                    #[external(v0)]
                    fn call_libfunc_syscall(
                        self: @ContractState, class_hash: felt252, function_selector: felt252, calldata: Span<felt252>
                    ) -> Span<felt252> {
                        library_call_syscall(
                            class_hash_try_from_felt252(class_hash).unwrap(), function_selector, calldata
                        )
                            .unwrap()
                    }
                }
                "#
            )
        ),
        Contract::new(
            "Executor",
            indoc!(
                r#"
                #[starknet::contract]
                mod Executor {
                    #[storage]
                    struct Storage {}
                
                    #[external(v0)]
                    fn add_two(self: @ContractState, number: felt252) -> felt252 {
                        number + 2
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
