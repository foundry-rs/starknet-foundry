use crate::common::corelib::corelib;
use crate::common::runner::Contract;
use crate::{assert_failed, assert_passed, test_case};
use camino::Utf8PathBuf;
use forge::run;
use forge::test_unit_summary::TestUnitSummary;
use indoc::indoc;
use std::path::Path;

#[test]
fn simple() {
    let test = test_case!(indoc!(
        r#"#[test]
        fn test_two_and_two() {
            assert(2 == 2, '2 == 2');
        }
    "#
    ));

    let result = run(
        &test.path().unwrap(),
        Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

#[test]
fn failing() {
    let test = test_case!(indoc!(
        r#"#[test]
        fn test_two_and_three() {
            assert(2 == 3, '2 == 3');
        }
    "#
    ));

    let result = run(
        &test.path().unwrap(),
        Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
    )
    .unwrap();

    assert_failed!(result);
}

#[test]
fn simple_declare() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;

        #[test]
        fn test_declare_simple() {
            assert(1 == 1, 'simple check');
            let class_hash = declare('HelloStarknet').unwrap();
            assert(class_hash != 0, 'proper class hash');
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
                    struct Storage {
                        balance: felt252,
                    }
        
                    // Increases the balance by the given amount.
                    #[external(v0)]
                    fn increase_balance(ref self: ContractState, amount: felt252) {
                        self.balance.write(self.balance.read() + amount);
                    }
        
                    // Decreases the balance by the given amount.
                    #[external(v0)]
                    fn decrease_balance(ref self: ContractState, amount: felt252) {
                        self.balance.write(self.balance.read() - amount);
                    }
                }
                "#
            )
        )
    );

    let result = run(
        &test.path().unwrap(),
        Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

#[test]
fn multiple_declare() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;

        #[test]
        fn multiple_contracts() {
            let class_hash = declare('HelloStarknet').unwrap();
            assert(class_hash != 0, 'proper class hash');
        
            let class_hash2 = declare('Contract1').unwrap();
            assert(class_hash2 != 0, 'proper class hash');
        
            assert(class_hash != class_hash2, 'class hashes neq');
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
                    struct Storage {
                        balance: felt252,
                    }
        
                    // Increases the balance by the given amount.
                    #[external(v0)]
                    fn increase_balance(ref self: ContractState, amount: felt252) {
                        self.balance.write(self.balance.read() + amount);
                    }
        
                    // Decreases the balance by the given amount.
                    #[external(v0)]
                    fn decrease_balance(ref self: ContractState, amount: felt252) {
                        self.balance.write(self.balance.read() - amount);
                    }
                }
                "#
            )
        ),
        Contract::new(
            "Contract1",
            indoc!(
                r#"
                #[starknet::interface]
                trait IContract1<TContractState> {
                    fn increase_balance(ref self: TContractState, amount: felt252);
                    fn get_balance(self: @TContractState) -> felt252;
                }
                
                #[starknet::contract]
                mod Contract1 {
                    #[storage]
                    struct Storage {
                        balance: felt252,
                    }
                
                    #[external(v0)]
                    impl Contract1Impl of super::IContract1<ContractState> {
                        // Increases the balance by the given amount.
                        fn increase_balance(ref self: ContractState, amount: felt252) {
                            assert(amount != 0, 'Amount cannot be 0');
                            self.balance.write(self.balance.read() + amount);
                        }
                
                        // Returns the current balance.
                        fn get_balance(self: @ContractState) -> felt252 {
                            self.balance.read()
                        }
                    }
                }
                "#
            )
        )
    );

    let result = run(
        &test.path().unwrap(),
        Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

#[test]
fn simple_declare_from_contract_code() {
    let contract = Contract::from_code_path(
        "Contract1".to_string(),
        Path::new("./tests/data/declare_test/src/contract1.cairo"),
    )
    .unwrap();

    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;

        #[test]
        fn test_declare_simple() {
            assert(1 == 1, 'simple check');
            let class_hash = declare('Contract1').unwrap();
            assert(class_hash != 0, 'proper class hash');
        }
        "#
        ),
        contract
    );

    let result = run(
        &test.path().unwrap(),
        Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}

#[test]
fn declare_unknown() {
    let test = test_case!(indoc!(
        r#"
        use result::ResultTrait;

        #[test]
        fn test_declare_simple() {
            assert(1 == 1, 'simple check');
            let class_hash = declare('Unknown').unwrap();
            assert(class_hash != 0, 'proper class hash');
        }
        "#
    ));

    let result = run(
        &test.path().unwrap(),
        Some(test.linked_libraries()),
        &Default::default(),
        Some(&Utf8PathBuf::from_path_buf(corelib().to_path_buf()).unwrap()),
        &test.contracts(corelib().path()).unwrap(),
    )
    .unwrap();

    assert_failed!(result);
}
