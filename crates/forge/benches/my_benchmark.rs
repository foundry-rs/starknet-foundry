use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use indoc::indoc;
use std::time::Duration;
use utils::runner::{Contract, TestCase};
use utils::running_tests::run_test_case;
use utils::{assert_passed, test_case};

fn setup_declare_and_interact() -> TestCase {
    test_case!(
        indoc!(
            r#"
        use result::ResultTrait;
        use traits::Into;
        use starknet::ClassHashIntoFelt252;
        use starknet::ContractAddress;
        use starknet::Felt252TryIntoContractAddress;
        use snforge_std::{ declare, ContractClassTrait, start_prank, start_roll, start_warp };

        #[starknet::interface]
        trait IHelloStarknet<TContractState> {
            fn increase_balance(ref self: TContractState, amount: felt252);
            fn decrease_balance(ref self: TContractState, amount: felt252);
            fn get_balance(self: @TContractState) -> felt252;
            fn interact_with_state(self: @TContractState) -> (felt252, felt252, felt252);
        }

        #[test]
        fn declare_and_interact() {
            assert(1 == 1, 'simple check');
            let contract = declare('HelloStarknet');
            let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher = IHelloStarknetDispatcher { contract_address };

            let balance = dispatcher.get_balance();
            dispatcher.increase_balance(100);
            let balance = dispatcher.get_balance();
            dispatcher.decrease_balance(100);
            let balance = dispatcher.get_balance();

            start_prank(contract_address, 1234.try_into().unwrap());
            start_roll(contract_address, 234);
            start_warp(contract_address, 123);

            let (x, y, z) = dispatcher.interact_with_state();
        }
        "#
        ),
        Contract::new(
            "HelloStarknet",
            indoc!(
                r#"
                #[starknet::contract]
                mod HelloStarknet {
                    use box::BoxTrait;
                    use starknet::ContractAddressIntoFelt252;
                    use starknet::ContractAddress;
                    use integer::U64IntoFelt252;
                    use option::Option;
                    use traits::Into;

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

                    #[external(v0)]
                    fn interact_with_state(self: @ContractState) -> (felt252, felt252, felt252) {
                        let caller_address: felt252 = starknet::get_caller_address().into();
                        let block_number = starknet::get_block_info().unbox().block_number;
                        let block_timestamp = starknet::get_block_info().unbox().block_timestamp;

                        (caller_address, block_number.into(), block_timestamp.into())
                    }
                }
                "#
            )
        )
    )
}

fn declare_and_interact(test: &TestCase) {
    let result = run_test_case(test);

    assert_passed!(result);
}

fn criterion_benchmark(c: &mut Criterion) {
    let test = setup_declare_and_interact();

    let mut group = c.benchmark_group("forge-benchmark-group");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(30));
    group.bench_with_input(
        BenchmarkId::new("declare_and_interact", format!("{test:?}")),
        &test,
        |b, s| b.iter(|| declare_and_interact(s)),
    );
    group.finish();
}

// TODO benchmark test compilation

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
