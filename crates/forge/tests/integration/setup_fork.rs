use indoc::formatdoc;
use std::path::Path;
use std::path::PathBuf;
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

use forge::compiled_raw::RawForkParams;
use forge_runner::{RunnerConfig, RunnerParams};
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_case_output_contains, assert_failed, assert_passed, test_case};

static INTEGRATION_RPC_URL: &str = "http://188.34.188.184:9545/rpc/v0_6";
static TESTNET_RPC_URL: &str = "http://188.34.188.184:6060/rpc";

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
            #[fork(url: "{}", block_id: BlockId::Number(313388))]
            fn fork_simple_decorator() {{
                let dispatcher = IHelloStarknetDispatcher {{
                    contract_address: contract_address_const::<3216637956526895219277698311134811322769343974163380838558193911733621219342>()
                }};

                let balance = dispatcher.get_balance();
                assert(balance == 2, 'Balance should be 2');

                dispatcher.increase_balance(100);

                let balance = dispatcher.get_balance();
                assert(balance == 102, 'Balance should be 102');
            }}
        "#,
        INTEGRATION_RPC_URL
    ).as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
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
                    contract_address: contract_address_const::<3216637956526895219277698311134811322769343974163380838558193911733621219342>()
                }};

                let balance = dispatcher.get_balance();
                assert(balance == 2, 'Balance should be 2');

                dispatcher.increase_balance(100);

                let balance = dispatcher.get_balance();
                assert(balance == 102, 'Balance should be 102');
            }}
        "#
    ).as_str());

    let rt = Runtime::new().expect("Could not instantiate Runtime");

    let test_build_output = Command::new("scarb")
        .current_dir(test.path().unwrap())
        .arg("snforge-test-collector")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    assert!(test_build_output.status.success());

    let result = rt
        .block_on(run(
            &String::from("test_package"),
            &test.path().unwrap().join("target/dev/snforge"),
            &TestsFilter::from_flags(None, false, false, false, false, Default::default()),
            Arc::new(RunnerConfig::new(
                Utf8PathBuf::from_path_buf(PathBuf::from(tempdir().unwrap().path())).unwrap(),
                false,
                256,
                12345,
            )),
            Arc::new(RunnerParams::new(
                test.contracts().unwrap(),
                test.env().clone(),
            )),
            &[ForkTarget::new(
                "FORK_NAME_FROM_SCARB_TOML".to_string(),
                RawForkParams {
                    url: INTEGRATION_RPC_URL.to_string(),
                    block_id_type: "Tag".to_string(),
                    block_id_value: "Latest".to_string(),
                },
            )],
            &mut BlockNumberMap::default(),
        ))
        .expect("Runner fail");

    assert_passed!(result);
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
            #[fork(url: "{}", block_id: BlockId::Number(313494))]
            fn fork_cairo0_contract() {{
                let contract_address = contract_address_const::<0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7>();

                let dispatcher = IERC20CamelDispatcher {{ contract_address }};

                let total_supply = dispatcher.totalSupply();
                assert(total_supply == 1368798332311330795498, 'Wrong total supply');
            }}
        "#,
        INTEGRATION_RPC_URL
    ).as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
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
            #[fork(url: "{INTEGRATION_RPC_URL}", block_id: BlockId::Number(315887))]
            fn test_fork_get_block_info_contract_on_testnet() {{
                let dispatcher = IBlockInfoCheckerDispatcher {{
                    contract_address: contract_address_const::<0x4bc9a2c302d2c704dbabe8fe396d9fe7b9ca65a46a3cf5d2edc6c57bddcf316>()
                }};

                let timestamp = dispatcher.read_block_timestamp();
                assert(timestamp == 1697630072, timestamp.into());
                let block_number = dispatcher.read_block_number();
                assert(block_number == 315887, block_number.into());

                let expected_sequencer_addr = contract_address_const::<0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8>();
                let sequencer_addr = dispatcher.read_sequencer_address();
                assert(sequencer_addr == expected_sequencer_addr, sequencer_addr.into());
            }}

            #[test]
            #[fork(url: "{INTEGRATION_RPC_URL}", block_id: BlockId::Number(315887))]
            fn test_fork_get_block_info_test_state() {{
                let block_info = starknet::get_block_info().unbox();
                assert(block_info.block_timestamp == 1697630072, block_info.block_timestamp.into());
                assert(block_info.block_number == 315887, block_info.block_number.into());
                let expected_sequencer_addr = contract_address_const::<0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8>();
                assert(block_info.sequencer_address == expected_sequencer_addr, block_info.sequencer_address.into());
            }}

            #[test]
            #[fork(url: "{INTEGRATION_RPC_URL}", block_id: BlockId::Number(315887))]
            fn test_fork_get_block_info_contract_deployed() {{
                let contract = declare('BlockInfoChecker');
                let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
                let dispatcher = IBlockInfoCheckerDispatcher {{ contract_address }};

                let timestamp = dispatcher.read_block_timestamp();
                assert(timestamp == 1697630072, timestamp.into());
                let block_number = dispatcher.read_block_number();
                assert(block_number == 315887, block_number.into());

                let expected_sequencer_addr = contract_address_const::<0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8>();
                let sequencer_addr = dispatcher.read_sequencer_address();
                assert(sequencer_addr == expected_sequencer_addr, sequencer_addr.into());
            }}

            #[test]
            #[fork(url: "{INTEGRATION_RPC_URL}", block_id: BlockId::Tag(BlockTag::Latest))]
            fn test_fork_get_block_info_latest_block() {{
                let block_info = starknet::get_block_info().unbox();
                assert(block_info.block_timestamp > 1697630072, block_info.block_timestamp.into());
                assert(block_info.block_number > 315887, block_info.block_number.into());
            }}
        "#
    ).as_str(),
    Contract::from_code_path(
        "BlockInfoChecker".to_string(),
        Path::new("tests/data/contracts/block_info_checker.cairo"),
    ).unwrap());

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn fork_get_block_info_fails() {
    let test = test_case!(formatdoc!(
        r#"
            #[test]
            #[fork(url: "{INTEGRATION_RPC_URL}", block_id: BlockId::Number(999999999999))]
            fn fork_get_block_info_fails() {{
                starknet::get_block_info().unbox();
            }}
        "#
    )
    .as_str());

    let result = run_test_case(&test);

    assert_failed!(result);
    assert_case_output_contains!(
        result,
        "fork_get_block_info_fails",
        "Unable to get block with tx hashes from fork"
    );
}

#[test]
// found in: https://github.com/foundry-rs/starknet-foundry/issues/1175
fn incompatible_abi() {
    let test = test_case!(formatdoc!(
        r#"
            #[derive(Serde)]
            struct Propdetails {{
                payload: felt252,
            }}

            #[starknet::interface]
            trait IGovernance<State> {{
                fn get_proposal_details(self: @State, param: felt252) -> Propdetails;
            }}

            #[test]
            #[fork(url: "{TESTNET_RPC_URL}", block_id: BlockId::Number(904597))]
            fn test_forking_functionality() {{
                let gov_contract_addr: starknet::ContractAddress = 0x7ba1d4836a1142c09dde23cb39b2885fe350912591461b5764454a255bdbac6.try_into().unwrap();
                let dispatcher = IGovernanceDispatcher {{ contract_address: gov_contract_addr }};
                let propdetails = dispatcher.get_proposal_details(1);
                assert(propdetails.payload==0x78b4ccacdc1c902281f6f13d94b6d17b1f4c44ff811c01dea504d43a264f611, 'payload not match');
            }}
        "#,
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
}
