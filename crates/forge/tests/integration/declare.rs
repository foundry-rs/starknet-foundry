use indoc::indoc;
use std::path::Path;
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_passed, test_case};

#[test]
fn simple_declare() {
    let test = test_case!(
        indoc!(
            r"
        use result::ResultTrait;
        use traits::Into;
        use starknet::ClassHashIntoFelt252;
        use snforge_std::declare;

        #[test]
        fn simple_declare() {
            assert(1 == 1, 'simple check');
            let contract = declare('HelloStarknet');
            assert(contract.class_hash.into() != 0, 'proper class hash');
        }
        "
        ),
        Contract::new(
            "HelloStarknet",
            indoc!(
                r"
                #[starknet::interface]
                trait IHelloStarknet<TContractState> {
                    fn increase_balance(ref self: TContractState, amount: felt252);
                    fn decrease_balance(ref self: TContractState, amount: felt252);
                }

                #[starknet::contract]
                mod HelloStarknet {
                    #[storage]
                    struct Storage {
                        balance: felt252,
                    }
        
                    // Increases the balance by the given amount.
                    #[abi(embed_v0)]
                    impl HelloStarknetImpl of super::IHelloStarknet<ContractState> {
                        fn increase_balance(ref self: ContractState, amount: felt252) {
                            self.balance.write(self.balance.read() + amount);
                        }

                        // Decreases the balance by the given amount.
                        fn decrease_balance(ref self: ContractState, amount: felt252) {
                            self.balance.write(self.balance.read() - amount);
                        }
                    }
                }
                "
            )
        )
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn declare_simple() {
    let contract = Contract::from_code_path(
        "Contract1".to_string(),
        Path::new("tests/data/contracts/hello_starknet.cairo"),
    )
    .unwrap();

    let test = test_case!(
        indoc!(
            r"
        use result::ResultTrait;
        use traits::Into;
        use starknet::ClassHashIntoFelt252;
        use snforge_std::declare;

        #[test]
        fn declare_simple() {
            assert(1 == 1, 'simple check');
            let contract = declare('Contract1');
            let class_hash = contract.class_hash.into();
            assert(class_hash != 0, 'proper class hash');
        }
        "
        ),
        contract
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}
