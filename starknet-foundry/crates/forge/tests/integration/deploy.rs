use crate::common::corelib::{corelib, predeployed_contracts};
use crate::common::runner::Contract;
use crate::{assert_case_output_contains, assert_failed, assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;
use std::path::Path;

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

#[test]
fn deploy_fails_on_calldata_when_contract_has_no_constructor() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use cheatcodes::RevertedTransactionTrait;
        use cheatcodes::PreparedContract;
        use array::ArrayTrait; 
            
        #[test]
        fn deploy_invalid_calldata() {
            let mut calldata = ArrayTrait::new();
            calldata.append(1234);
            calldata.append(5678);
        
            let class_hash = declare('HelloStarknet').unwrap();
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();
        
            assert(2 == 2, '2 == 2');
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
        &Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_failed!(result);
}

#[test]
fn test_deploy_fails_on_missing_constructor_arguments() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use cheatcodes::RevertedTransactionTrait;
        use cheatcodes::PreparedContract;
        use array::ArrayTrait; 
            
        #[test]
        fn deploy_invalid_calldata() {
            let mut calldata = ArrayTrait::new();
        
            let class_hash = declare('HelloStarknet').unwrap();
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();
        
            assert(2 == 2, '2 == 2');
        }
    "#
        ),
        Contract::new(
            "HelloStarknet",
            indoc!(
                r#"
                #[starknet::contract]
                mod HelloStarknet {
                    #[storage]
                    struct Storage {}

                    #[constructor]
                    fn constructor(ref self: ContractState, arg1: felt252, arg2: felt252) {}
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

    assert_failed!(result);
}

#[test]
fn test_deploy_fails_on_too_many_constructor_arguments() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use cheatcodes::RevertedTransactionTrait;
        use cheatcodes::PreparedContract;
        use array::ArrayTrait;

        #[test]
        fn deploy_invalid_calldata() {
            let mut calldata = ArrayTrait::new();
            calldata.append(1);
            calldata.append(2);
            calldata.append(3);
            calldata.append(4);
            calldata.append(5);

            let class_hash = declare('HelloStarknet').unwrap();
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();

            assert(2 == 2, '2 == 2');
        }
    "#
        ),
        Contract::new(
            "HelloStarknet",
            indoc!(
                r#"
                #[starknet::contract]
                mod HelloStarknet {
                    #[storage]
                    struct Storage {}

                    #[constructor]
                    fn constructor(ref self: ContractState, arg1: felt252, arg2: felt252) {}
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

    assert_failed!(result);
}

#[test]
fn test_deploy_fails_with_incorrect_class_hash() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use cheatcodes::RevertedTransactionTrait;
        use cheatcodes::PreparedContract;
        use array::ArrayTrait;

        #[test]
        fn deploy_not_existing_class_hash() {
            let mut calldata = ArrayTrait::new();

            let prepared = PreparedContract { class_hash: 'made-up-class-hash', constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();
        }
    "#
        ),
        Contract::new(
            "HelloStarknet",
            indoc!(
                r#"
                #[starknet::contract]
                mod HelloStarknet {
                    #[storage]
                    struct Storage {}
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

    assert_case_output_contains!(result, "deploy_not_existing_class_hash", "not declared");
    assert_failed!(result);
}

#[test]
fn test_deploy_invokes_the_constructor() {
    let test = test_case!(
        indoc!(
            r#"
        use option::OptionTrait;
        use result::ResultTrait;
        use cheatcodes::RevertedTransactionTrait;
        use cheatcodes::PreparedContract;
        use array::ArrayTrait;
        use traits::TryInto;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        
        #[starknet::interface]
        trait ThingGetter<TContractState> {
            fn get_thing(self: @TContractState) -> felt252;
        }
        
        #[test]
        fn deploy_invokes_constructor() {
            let mut calldata = ArrayTrait::new();
            calldata.append(420);

            let class_hash = declare('HelloStarknet').unwrap();
            let prepared = PreparedContract { class_hash: class_hash, constructor_calldata: @calldata };
            let contract_address = deploy(prepared).unwrap();
            let contract_address: ContractAddress = contract_address.try_into().unwrap();
            
            let thing_getter = ThingGetterDispatcher { contract_address };
            
            let thing = thing_getter.get_thing();
            
            assert(thing == 420, 'Incorrect thing');
        }
    "#
        ),
        Contract::new(
            "HelloStarknet",
            indoc!(
                r#"
                #[starknet::interface]
                trait ThingGetter<TContractState> {
                    fn get_thing(self: @TContractState) -> felt252;
                }
                
                #[starknet::contract]
                mod HelloStarknet {
                    #[storage]
                    struct Storage {
                        stored_thing: felt252
                    }
                    #[constructor]
                     fn constructor(ref self: ContractState, arg1: felt252) {
                        self.stored_thing.write(arg1)
                     }
                     
                     #[external(v0)]
                     impl ThingGetterImpl of super::ThingGetter<ContractState> {
                        fn get_thing(self: @ContractState) -> felt252 {
                            self.stored_thing.read()
                        }
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
