use crate::integration::common::running_tests::run_test_case;
use crate::{assert_passed, test_case};

use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use camino::Utf8PathBuf;
use forge::scarb::{ForgeConfig, ForkTarget};
use forge::{run, CancellationTokens, RunnerConfig, RunnerParams};
use indoc::formatdoc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tempfile::tempdir;
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
            Arc::new(RunnerConfig::new(
                Utf8PathBuf::from_path_buf(PathBuf::from(tempdir().unwrap().path())).unwrap(),
                None,
                false,
                false,
                Some(1234),
                Some(500),
                &ForgeConfig {
                    exit_first: false,
                    fuzzer_runs: Some(1234),
                    fuzzer_seed: Some(500),
                    fork: vec![ForkTarget {
                        name: "FORK_NAME_FROM_SCARB_TOML".to_string(),
                        url: CHEATNET_RPC_URL.to_string(),
                        block_id: HashMap::from([("tag".to_string(), "Latest".to_string())]),
                    }],
                },
            )),
            Arc::new(RunnerParams::new(
                corelib_path(),
                test.contracts(&corelib_path()).unwrap(),
                Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
                Default::default(),
                test.linked_libraries(),
            )),
            Arc::new(CancellationTokens::new()),
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
