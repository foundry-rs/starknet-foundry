use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use camino::Utf8PathBuf;
use forge::scarb::config::ForkTarget;
use forge::test_filter::TestsFilter;
use forge::{run, RunnerConfig, RunnerParams};
use indoc::formatdoc;
use starknet::core::types::BlockId;
use starknet::core::types::BlockTag::Latest;
use tempfile::tempdir;
use test_collector::RawForkParams;
use test_utils::corelib::{corelib_path, predeployed_contracts};
use test_utils::runner::Contract;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_case_output_contains, assert_failed, assert_passed, test_case};
use tokio::runtime::Runtime;

static CHEATNET_RPC_URL: &str = "http://188.34.188.184:9545/rpc/v0.4";

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
            fn test_fork_simple() {{
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
        CHEATNET_RPC_URL
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
            fn test_fork_simple() {{
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

    let result = rt
        .block_on(run(
            &test.path().unwrap(),
            &String::from("src"),
            &test.path().unwrap().join("src"),
            &TestsFilter::from_flags(None, false, false, false),
            Arc::new(RunnerConfig::new(
                Utf8PathBuf::from_path_buf(PathBuf::from(tempdir().unwrap().path())).unwrap(),
                false,
                vec![ForkTarget {
                    name: "FORK_NAME_FROM_SCARB_TOML".to_string(),
                    params: RawForkParams {
                        url: CHEATNET_RPC_URL.to_string(),
                        block_id: BlockId::Tag(Latest),
                    },
                }],
                256,
                12345,
            )),
            Arc::new(RunnerParams::new(
                corelib_path(),
                test.contracts(&corelib_path()).unwrap(),
                Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
                Default::default(),
                test.linked_libraries(),
            )),
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
            fn test_timestamp() {{
                let contract_address = contract_address_const::<0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7>();

                let dispatcher = IERC20CamelDispatcher {{ contract_address }};

                let total_supply = dispatcher.totalSupply();
                assert(total_supply == 1368798332311330795498, 'Wrong total supply');
            }}
        "#,
        CHEATNET_RPC_URL
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
            #[fork(url: "{CHEATNET_RPC_URL}", block_id: BlockId::Number(315887))]
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
            #[fork(url: "{CHEATNET_RPC_URL}", block_id: BlockId::Number(315887))]
            fn test_fork_get_block_info_test_state() {{
                let block_info = starknet::get_block_info().unbox();
                assert(block_info.block_timestamp == 1697630072, block_info.block_timestamp.into());
                assert(block_info.block_number == 315887, block_info.block_number.into());
                let expected_sequencer_addr = contract_address_const::<0x1176a1bd84444c89232ec27754698e5d2e7e1a7f1539f12027f28b23ec9f3d8>();
                assert(block_info.sequencer_address == expected_sequencer_addr, block_info.sequencer_address.into());
            }}

            #[test]
            #[fork(url: "{CHEATNET_RPC_URL}", block_id: BlockId::Number(315887))]
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
            #[fork(url: "{CHEATNET_RPC_URL}", block_id: BlockId::Tag(BlockTag::Latest))]
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
fn test_fork_get_block_info_fails() {
    let test = test_case!(formatdoc!(
        r#"
            #[test]
            #[fork(url: "{CHEATNET_RPC_URL}", block_id: BlockId::Number(999999999999))]
            fn test_fork_get_block_info_fails() {{
                let block_info = starknet::get_block_info().unbox();
            }}
        "#
    )
    .as_str());

    let result = run_test_case(&test);

    assert_failed!(result);
    assert_case_output_contains!(
        result,
        "test_fork_get_block_info_fails",
        "Unable to get block with tx hashes from fork"
    );
}
