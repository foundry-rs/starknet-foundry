use criterion::{black_box, criterion_group, criterion_main, Criterion};
use indoc::indoc;
use utils::runner::Contract;
use utils::running_tests::run_test_case;
use utils::{assert_passed, test_case};

fn simple_declare() {
    let test = test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use traits::Into;
        use starknet::ClassHashIntoFelt252;
        use snforge_std::declare;

        #[test]
        fn test_declare_simple() {
            assert(1 == 1, 'simple check');
            let contract = declare('HelloStarknet');
            assert(contract.class_hash.into() != 0, 'proper class hash');
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

    let result = run_test_case(&test);

    assert_passed!(result);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("declare_simple", |b| b.iter(simple_declare));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
