use assert_fs::fixture::PathCopy;
use assert_fs::TempDir;
use camino::Utf8PathBuf;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, SamplingMode};
use forge::collecting::{collect_test_compilation_targets, TestCompilationTarget};
use forge::CrateLocation;
use indoc::indoc;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use test_collector::LinkedLibrary;
use test_utils::corelib::corelib_path;
use test_utils::runner::{Contract, TestCase};
use test_utils::running_tests::run_test_case;
use test_utils::{assert_passed, test_case};

fn setup_collect_tests() -> TempDir {
    let temp = TempDir::new().unwrap();
    temp.copy_from("tests/data/simple_package", &["**/*.cairo", "**/*.toml"])
        .unwrap();

    temp
}

fn collect_tests(package: &TempDir) {
    let path = Utf8PathBuf::from_path_buf(package.to_path_buf()).unwrap();

    let _ = collect_test_compilation_targets(&path, "simple_package", &path);
}

fn setup_compile_tests() -> (
    TestCompilationTarget,
    Vec<LinkedLibrary>,
    Utf8PathBuf,
    TempDir,
) {
    let package = setup_collect_tests();
    let path = Utf8PathBuf::from_path_buf(package.to_path_buf())
        .unwrap()
        .join("src");

    let snforge_std_path = PathBuf::from_str("../../snforge_std")
        .unwrap()
        .canonicalize()
        .unwrap();
    let linked_libraries = vec![
        LinkedLibrary {
            name: "simple_package".to_string(),
            path: PathBuf::from(path.clone()),
        },
        LinkedLibrary {
            name: "snforge_std".to_string(),
            path: snforge_std_path.join("src"),
        },
    ];

    let lib_content = std::fs::read_to_string(path.join("lib.cairo")).unwrap();
    let test_crate = TestCompilationTarget {
        crate_root: path,
        crate_name: "simple_package".to_string(),
        crate_location: CrateLocation::Lib,
        lib_content,
    };

    (test_crate, linked_libraries, corelib_path(), package)
}

fn compile_tests(
    test_crate: &TestCompilationTarget,
    linked_libraries: &[LinkedLibrary],
    corelib_path: &Utf8PathBuf,
) {
    test_crate
        .compile_tests(linked_libraries, corelib_path)
        .unwrap();
}

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
    let collect_tests_input = setup_collect_tests();
    let compile_tests_input = setup_compile_tests();

    let mut group = c.benchmark_group("benchmark-normal-flow");
    group.sampling_mode(SamplingMode::Flat); // https://bheisler.github.io/criterion.rs/book/user_guide/advanced_configuration.html#sampling-mode
    group.sample_size(50);
    group.measurement_time(Duration::from_secs(120));
    group.bench_with_input(
        BenchmarkId::new("declare_and_interact", format!("{test:?}")),
        &test,
        |b, test_case| b.iter(|| declare_and_interact(test_case)),
    );
    group.bench_with_input(
        BenchmarkId::new("collect_tests", format!("{test:?}")),
        &collect_tests_input,
        |b, package| b.iter(|| collect_tests(package)),
    );
    group.bench_with_input(
        BenchmarkId::new("compile_tests", format!("{test:?}")),
        &compile_tests_input,
        |b, (compilation_target, linked_libraries, corelib_path, _)| {
            b.iter(|| compile_tests(compilation_target, linked_libraries, corelib_path));
        },
    );
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
