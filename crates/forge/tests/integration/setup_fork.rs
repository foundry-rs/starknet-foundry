use indoc::formatdoc;

use std::num::NonZeroU32;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;

use camino::Utf8PathBuf;
use forge::block_number_map::BlockNumberMap;
use forge::run;
use forge::scarb::config::ForkTarget;
use forge::test_filter::TestsFilter;
use tempfile::tempdir;
use tokio::runtime::Runtime;

use cheatnet::runtime_extensions::forge_runtime_extension::contracts_data::ContractsData;
use forge::compiled_raw::RawForkParams;
use forge::scarb::{get_test_artifacts_path, load_test_artifacts};
use forge_runner::forge_config::{
    ExecutionDataToSave, ForgeConfig, OutputConfig, SierraTestCodePathConfig, TestRunnerConfig,
};
use forge_runner::{CACHE_DIR, SIERRA_TEST_CODE_DIR};
use shared::command::CommandExt;
use test_utils::runner::{assert_case_output_contains, assert_failed, assert_passed, Contract};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

const TESTNET_RPC_URL: &str = "http://188.34.188.184:7070/rpc/v0_7";

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
            use snforge_std::{{ BlockTag, BlockId }};

            #[starknet::interface]
            trait IHelloStarknet<TContractState> {{
                fn increase_balance(ref self: TContractState, amount: felt252);
                fn get_balance(self: @TContractState) -> felt252;
            }}

            #[test]
            #[fork(url: "{}", block_id: BlockId::Number(54060))]
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
        TESTNET_RPC_URL
    ).as_str());

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn fork_aliased_decorator() {
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
            #[fork("FORK_NAME_FROM_SCARB_TOML")]
            fn fork_aliased_decorator() {{
                let dispatcher = IHelloStarknetDispatcher {{
                    contract_address: contract_address_const::<0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9>()
                }};

                let balance = dispatcher.get_balance();
                assert(balance == 0, 'Balance should be 0');

                dispatcher.increase_balance(100);

                let balance = dispatcher.get_balance();
                assert(balance == 100, 'Balance should be 100');
            }}
        "#
    ).as_str());

    let rt = Runtime::new().expect("Could not instantiate Runtime");

    Command::new("scarb")
        .current_dir(test.path().unwrap())
        .arg("snforge-test-collector")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output_checked()
        .unwrap();

    let test_artifacts_path = get_test_artifacts_path(
        &test.path().unwrap().join("target/dev/snforge"),
        "test_package",
    );
    let compiled_test_crates = load_test_artifacts(&test_artifacts_path).unwrap();

    let result = rt
        .block_on(run(
            compiled_test_crates,
            "test_package",
            &TestsFilter::from_flags(None, false, false, false, false, Default::default()),
            Arc::new(ForgeConfig {
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
                    execution_data_to_save: ExecutionDataToSave::None,
                }),
                sierra_test_code_path_config: SierraTestCodePathConfig {
                    package_name: "test_package".to_string(),
                    sierra_test_code_dir: Utf8PathBuf::from_path_buf(
                        tempdir().unwrap().into_path(),
                    )
                    .unwrap()
                    .join(SIERRA_TEST_CODE_DIR),
                },
            }),
            &[ForkTarget::new(
                "FORK_NAME_FROM_SCARB_TOML".to_string(),
                RawForkParams {
                    url: TESTNET_RPC_URL.to_string(),
                    block_id_type: "Tag".to_string(),
                    block_id_value: "Latest".to_string(),
                },
            )],
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
            #[fork(url: "{}", block_id: BlockId::Number(54060))]
            fn fork_cairo0_contract() {{
                let contract_address = contract_address_const::<0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7>();

                let dispatcher = IERC20CamelDispatcher {{ contract_address }};

                let total_supply = dispatcher.totalSupply();
                assert(total_supply == 88730316280408105750094, 'Wrong total supply');
            }}
        "#,
        TESTNET_RPC_URL
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
            use snforge_std::{{ BlockTag, BlockId, declare, ContractClassTrait }};

            #[starknet::interface]
            trait IBlockInfoChecker<TContractState> {{
                fn read_block_number(self: @TContractState) -> u64;
                fn read_block_timestamp(self: @TContractState) -> u64;
                fn read_sequencer_address(self: @TContractState) -> ContractAddress;
            }}

            #[test]
            #[fork(url: "{TESTNET_RPC_URL}", block_id: BlockId::Number(54060))]
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
            #[fork(url: "{TESTNET_RPC_URL}", block_id: BlockId::Number(54060))]
            fn test_fork_get_block_info_test_state() {{
                let block_info = starknet::get_block_info().unbox();
                assert(block_info.block_timestamp == 1711645884, block_info.block_timestamp.into());
                assert(block_info.block_number == 54060, block_info.block_number.into());
                let expected_sequencer_addr = contract_address_const::<0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8>();
                assert(block_info.sequencer_address == expected_sequencer_addr, block_info.sequencer_address.into());
            }}

            #[test]
            #[fork(url: "{TESTNET_RPC_URL}", block_id: BlockId::Number(54060))]
            fn test_fork_get_block_info_contract_deployed() {{
                let contract = declare("BlockInfoChecker").unwrap();
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
            #[fork(url: "{TESTNET_RPC_URL}", block_id: BlockId::Tag(BlockTag::Latest))]
            fn test_fork_get_block_info_latest_block() {{
                let block_info = starknet::get_block_info().unbox();
                assert(block_info.block_timestamp > 1711645884, block_info.block_timestamp.into());
                assert(block_info.block_number > 54060, block_info.block_number.into());
            }}
        "#
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
            #[fork(url: "{TESTNET_RPC_URL}", block_id: BlockId::Number(999999999999))]
            fn fork_get_block_info_fails() {{
                starknet::get_block_info();
            }}
        "#
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
            #[fork(url: "{TESTNET_RPC_URL}", block_id: BlockId::Tag(BlockTag::Latest))]
            fn test_forking_functionality() {{
                let gov_contract_addr: starknet::ContractAddress = 0x66e4b798c66160bd5fd04056938e5c9f65d67f183dfab9d7d0d2ed9413276fe.try_into().unwrap();
                let dispatcher = IResponseWith2FeltsDispatcher {{ contract_address: gov_contract_addr }};
                let propdetails = dispatcher.get();
                assert(propdetails.payload == 8, 'payload not match');
            }}
        "#,
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed(&result);
}
