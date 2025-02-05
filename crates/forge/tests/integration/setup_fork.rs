use cheatnet::runtime_extensions::forge_config_extension::config::BlockId;
use indoc::{formatdoc, indoc};
use std::num::NonZeroU32;
use std::path::Path;
use std::sync::Arc;

use camino::Utf8PathBuf;
use forge::block_number_map::BlockNumberMap;
use forge::run_tests::package::run_for_package;
use forge::scarb::config::ForkTarget;
use forge::shared_cache::FailedTestsCache;
use forge::test_filter::TestsFilter;
use tempfile::tempdir;
use tokio::runtime::Runtime;

use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use forge::run_tests::package::RunForPackageArgs;
use forge::scarb::load_test_artifacts;
use forge_runner::forge_config::{
    ExecutionDataToSave, ForgeConfig, OutputConfig, TestRunnerConfig,
};
use forge_runner::CACHE_DIR;
use scarb_api::metadata::MetadataCommandExt;
use scarb_api::ScarbCommand;
use shared::test_utils::node_url::node_rpc_url;
use test_utils::runner::{assert_case_output_contains, assert_failed, assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn fork_simple_decorator() {
    let test = test_case!(formatdoc!(
        r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use starknet::contract_address_const;

            #[starknet::interface]
            trait IHelloStarknet<TContractState> {{
                fn increase_balance(ref self: TContractState, amount: felt252);
                fn get_balance(self: @TContractState) -> felt252;
            }}

            #[test]
            #[fork(url: "{}", block_number: 54060)]
            fn fork_simple_decorator() {{
                let dispatcher = IHelloStarknetDispatcher {{
                    contract_address: contract_address_const::<0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9>()
                }};

                let balance = dispatcher.get_balance();
                assert(balance == 0, 'Balance should be 0');

                dispatcher.increase_balance(100);

                let balance = dispatcher.get_balance();
                assert(balance == 100, 'Balance should be 100');
            }}
        "#,
        node_rpc_url()
    ).as_str());

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn fork_aliased_decorator() {
    let test = test_case!(indoc!(
        r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use starknet::contract_address_const;

            #[starknet::interface]
            trait IHelloStarknet<TContractState> {
                fn increase_balance(ref self: TContractState, amount: felt252);
                fn get_balance(self: @TContractState) -> felt252;
            }

            #[test]
            #[fork("FORK_NAME_FROM_SCARB_TOML")]
            fn fork_aliased_decorator() {
                let dispatcher = IHelloStarknetDispatcher {
                    contract_address: contract_address_const::<0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9>()
                };

                let balance = dispatcher.get_balance();
                assert(balance == 0, 'Balance should be 0');

                dispatcher.increase_balance(100);

                let balance = dispatcher.get_balance();
                assert(balance == 100, 'Balance should be 100');
            }
        "#
    ));

    let rt = Runtime::new().expect("Could not instantiate Runtime");

    ScarbCommand::new_with_stdio()
        .current_dir(test.path().unwrap())
        .arg("build")
        .arg("--test")
        .run()
        .unwrap();

    let metadata = ScarbCommand::metadata()
        .current_dir(test.path().unwrap())
        .run()
        .unwrap();

    let package = metadata
        .packages
        .iter()
        .find(|p| p.name == "test_package")
        .unwrap();

    let raw_test_targets =
        load_test_artifacts(&test.path().unwrap().join("target/dev"), package).unwrap();

    let result = rt
        .block_on(run_for_package(
            RunForPackageArgs {
                test_targets: raw_test_targets,
                package_name: "test_package".to_string(),
                tests_filter: TestsFilter::from_flags(
                    None,
                    false,
                    false,
                    false,
                    false,
                    FailedTestsCache::default(),
                ),
                forge_config: Arc::new(ForgeConfig {
                    test_runner_config: Arc::new(TestRunnerConfig {
                        exit_first: false,
                        fuzzer_runs: NonZeroU32::new(256).unwrap(),
                        fuzzer_seed: 12345,
                        max_n_steps: None,
                        is_vm_trace_needed: false,
                        cache_dir: Utf8PathBuf::from_path_buf(tempdir().unwrap().into_path())
                            .unwrap()
                            .join(CACHE_DIR),
                        contracts_data: ContractsData::try_from(test.contracts().unwrap()).unwrap(),
                        environment_variables: test.env().clone(),
                    }),
                    output_config: Arc::new(OutputConfig {
                        detailed_resources: false,
                        execution_data_to_save: ExecutionDataToSave::default(),
                    }),
                }),
                fork_targets: vec![ForkTarget {
                    name: "FORK_NAME_FROM_SCARB_TOML".to_string(),
                    url: node_rpc_url().as_str().parse().unwrap(),
                    block_id: BlockId::BlockTag,
                }],
            },
            &mut BlockNumberMap::default(),
        ))
        .expect("Runner fail");

    assert_passed(&result);
}

#[test]
fn fork_aliased_decorator_overrding() {
    let test = test_case!(indoc!(
        r#"
            use starknet::syscalls::get_execution_info_syscall;

            #[test]
            #[fork("FORK_NAME_FROM_SCARB_TOML", block_number: 2137)]
            fn test_get_block_number() {
                let execution_info = get_execution_info_syscall().unwrap().deref();
                let block_info = execution_info.block_info.deref();
                let block_number = block_info.block_number;

                assert(block_number == 2137, 'Invalid block');
            }
        "#
    ));

    let rt = Runtime::new().expect("Could not instantiate Runtime");

    ScarbCommand::new_with_stdio()
        .current_dir(test.path().unwrap())
        .arg("build")
        .arg("--test")
        .run()
        .unwrap();

    let metadata = ScarbCommand::metadata()
        .current_dir(test.path().unwrap())
        .run()
        .unwrap();

    let package = metadata
        .packages
        .iter()
        .find(|p| p.name == "test_package")
        .unwrap();

    let raw_test_targets =
        load_test_artifacts(&test.path().unwrap().join("target/dev"), package).unwrap();

    let result = rt
        .block_on(run_for_package(
            RunForPackageArgs {
                test_targets: raw_test_targets,
                package_name: "test_package".to_string(),
                tests_filter: TestsFilter::from_flags(
                    None,
                    false,
                    false,
                    false,
                    false,
                    FailedTestsCache::default(),
                ),
                forge_config: Arc::new(ForgeConfig {
                    test_runner_config: Arc::new(TestRunnerConfig {
                        exit_first: false,
                        fuzzer_runs: NonZeroU32::new(256).unwrap(),
                        fuzzer_seed: 12345,
                        max_n_steps: None,
                        is_vm_trace_needed: false,
                        cache_dir: Utf8PathBuf::from_path_buf(tempdir().unwrap().into_path())
                            .unwrap()
                            .join(CACHE_DIR),
                        contracts_data: ContractsData::try_from(test.contracts().unwrap()).unwrap(),
                        environment_variables: test.env().clone(),
                    }),
                    output_config: Arc::new(OutputConfig {
                        detailed_resources: false,
                        execution_data_to_save: ExecutionDataToSave::default(),
                    }),
                }),
                fork_targets: vec![ForkTarget {
                    name: "FORK_NAME_FROM_SCARB_TOML".to_string(),
                    url: node_rpc_url().as_str().parse().unwrap(),
                    block_id: BlockId::BlockNumber(12_341_234),
                }],
            },
            &mut BlockNumberMap::default(),
        ))
        .expect("Runner fail");

    assert_passed(&result);
}

#[test]
fn fork_cairo0_contract() {
    let test = test_case!(formatdoc!(
        r#"
            use starknet::contract_address_const;

            #[starknet::interface]
            trait IERC20Camel<TState> {{
                fn totalSupply(self: @TState) -> u256;
            }}

            #[test]
            #[fork(url: "{}", block_number: 54060)]
            fn fork_cairo0_contract() {{
                let contract_address = contract_address_const::<0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7>();

                let dispatcher = IERC20CamelDispatcher {{ contract_address }};

                let total_supply = dispatcher.totalSupply();
                assert(total_supply == 88730316280408105750094, 'Wrong total supply');
            }}
        "#,
        node_rpc_url()
    ).as_str());

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn get_block_info_in_forked_block() {
    let test = test_case!(formatdoc!(
        r#"
            use starknet::ContractAddress;
            use starknet::ContractAddressIntoFelt252;
            use starknet::contract_address_const;
            use snforge_std::{{ declare, ContractClassTrait, DeclareResultTrait }};

            #[starknet::interface]
            trait IBlockInfoChecker<TContractState> {{
                fn read_block_number(self: @TContractState) -> u64;
                fn read_block_timestamp(self: @TContractState) -> u64;
                fn read_sequencer_address(self: @TContractState) -> ContractAddress;
            }}

            #[test]
            #[fork(url: "{node_rpc_url}", block_number: 54060)]
            fn test_fork_get_block_info_contract_on_testnet() {{
                let dispatcher = IBlockInfoCheckerDispatcher {{
                    contract_address: contract_address_const::<0x3d80c579ad7d83ff46634abe8f91f9d2080c5c076d4f0f59dd810f9b3f01164>()
                }};

                let timestamp = dispatcher.read_block_timestamp();
                assert(timestamp == 1711645884, timestamp.into());
                let block_number = dispatcher.read_block_number();
                assert(block_number == 54060, block_number.into());

                let expected_sequencer_addr = contract_address_const::<0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8>();
                let sequencer_addr = dispatcher.read_sequencer_address();
                assert(sequencer_addr == expected_sequencer_addr, sequencer_addr.into());
            }}

            #[test]
            #[fork(url: "{node_rpc_url}", block_number: 54060)]
            fn test_fork_get_block_info_test_state() {{
                let block_info = starknet::get_block_info().unbox();
                assert(block_info.block_timestamp == 1711645884, block_info.block_timestamp.into());
                assert(block_info.block_number == 54060, block_info.block_number.into());
                let expected_sequencer_addr = contract_address_const::<0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8>();
                assert(block_info.sequencer_address == expected_sequencer_addr, block_info.sequencer_address.into());
            }}

            #[test]
            #[fork(url: "{node_rpc_url}", block_number: 54060)]
            fn test_fork_get_block_info_contract_deployed() {{
                let contract = declare("BlockInfoChecker").unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IBlockInfoCheckerDispatcher {{ contract_address }};

                let timestamp = dispatcher.read_block_timestamp();
                assert(timestamp == 1711645884, timestamp.into());
                let block_number = dispatcher.read_block_number();
                assert(block_number == 54060, block_number.into());

                let expected_sequencer_addr = contract_address_const::<0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8>();
                let sequencer_addr = dispatcher.read_sequencer_address();
                assert(sequencer_addr == expected_sequencer_addr, sequencer_addr.into());
            }}

            #[test]
            #[fork(url: "{node_rpc_url}", block_tag: latest)]
            fn test_fork_get_block_info_latest_block() {{
                let block_info = starknet::get_block_info().unbox();
                assert(block_info.block_timestamp > 1711645884, block_info.block_timestamp.into());
                assert(block_info.block_number > 54060, block_info.block_number.into());
            }}

            #[test]
            #[fork(url: "{node_rpc_url}", block_hash: 0x06ae121e46f5375f93b00475fb130348ae38148e121f84b0865e17542e9485de)]
            fn test_fork_get_block_info_block_hash() {{
                let block_info = starknet::get_block_info().unbox();
                assert(block_info.block_timestamp == 1711645884, block_info.block_timestamp.into());
                assert(block_info.block_number == 54060, block_info.block_number.into());
                let expected_sequencer_addr = contract_address_const::<0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8>();
                assert(block_info.sequencer_address == expected_sequencer_addr, block_info.sequencer_address.into());
            }}

            #[test]
            #[fork(url: "{node_rpc_url}", block_hash: 3021433528476416000728121069095289682281028310523383289416465162415092565470)]
            fn test_fork_get_block_info_block_hash_with_number() {{
                let block_info = starknet::get_block_info().unbox();
                assert(block_info.block_timestamp == 1711645884, block_info.block_timestamp.into());
                assert(block_info.block_number == 54060, block_info.block_number.into());
                let expected_sequencer_addr = contract_address_const::<0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8>();
                assert(block_info.sequencer_address == expected_sequencer_addr, block_info.sequencer_address.into());
            }}
        "#,
        node_rpc_url = node_rpc_url()
    ).as_str(),
    Contract::from_code_path(
        "BlockInfoChecker".to_string(),
        Path::new("tests/data/contracts/block_info_checker.cairo"),
    ).unwrap());

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn fork_get_block_info_fails() {
    let test = test_case!(formatdoc!(
        r#"
            #[test]
            #[fork(url: "{}", block_number: 999999999999)]
            fn fork_get_block_info_fails() {{
                starknet::get_block_info();
            }}
        "#,
        node_rpc_url()
    )
    .as_str());

    let result = run_test_case(&test);

    assert_failed(&result);
    assert_case_output_contains(
        &result,
        "fork_get_block_info_fails",
        "Unable to get block with tx hashes from fork",
    );
}

#[test]
// found in: https://github.com/foundry-rs/starknet-foundry/issues/1175
fn incompatible_abi() {
    let test = test_case!(formatdoc!(
        r#"
            #[derive(Serde)]
            struct Response {{
                payload: felt252,
                // there is second field on chain
            }}

            #[starknet::interface]
            trait IResponseWith2Felts<State> {{
                fn get(self: @State) -> Response;
            }}

            #[test]
            #[fork(url: "{}", block_tag: latest)]
            fn test_forking_functionality() {{
                let gov_contract_addr: starknet::ContractAddress = 0x66e4b798c66160bd5fd04056938e5c9f65d67f183dfab9d7d0d2ed9413276fe.try_into().unwrap();
                let dispatcher = IResponseWith2FeltsDispatcher {{ contract_address: gov_contract_addr }};
                let propdetails = dispatcher.get();
                assert(propdetails.payload == 8, 'payload not match');
            }}
        "#,
        node_rpc_url()
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed(&result);
}
