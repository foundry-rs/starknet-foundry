use crate::common::corelib::corelib;
use crate::{assert_passed, contract, test_case};
use camino::Utf8PathBuf;
use forge::run;
use indoc::indoc;

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
        contract!(
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
                    #[external]
                    fn increase_balance(ref self: ContractState, amount: felt252) {
                        self.balance.write(self.balance.read() + amount);
                    }
        
                    // Decreases the balance by the given amount.
                    #[external]
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
