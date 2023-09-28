use criterion::{criterion_group, criterion_main, Criterion};
use indoc::indoc;
use std::time::Duration;
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
        use snforge_std::{ declare, ContractClassTrait };

        #[starknet::interface]
        trait IHelloStarknet<TContractState> {
            fn increase_balance(ref self: TContractState, amount: felt252);
            fn decrease_balance(ref self: TContractState, amount: felt252);
            fn get_balance(self: @TContractState) -> felt252;
        }

        #[test]
        fn test_declare_simple() {
            assert(1 == 1, 'simple check');
            let contract = declare('HelloStarknet');
            let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher = IHelloStarknetDispatcher { contract_address };

            let balance = dispatcher.get_balance();
            dispatcher.increase_balance(100);
            let balance = dispatcher.get_balance();
            dispatcher.decrease_balance(100);
            let balance = dispatcher.get_balance();
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

                    //Get the balance.
                    #[external(v0)]
                    fn get_balance(self: @ContractState) -> felt252 {
                        self.balance.read()
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
    let mut group = c.benchmark_group("declare-group");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));
    group.bench_function("declare_simple", |b| b.iter(simple_declare));
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
