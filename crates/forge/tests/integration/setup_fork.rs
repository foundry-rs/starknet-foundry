use crate::integration::common::corelib::{corelib_path, predeployed_contracts};
use crate::integration::common::running_tests::run_test_case;
use crate::{assert_passed, test_case};
use camino::Utf8PathBuf;
use dotenv::dotenv;
use forge::scarb::{ForgeConfig, PredefinedFork};
use forge::{run, RunnerConfig};
use indoc::formatdoc;
use std::collections::HashMap;

#[test]
fn fork_simple_decorator() {
    dotenv().ok().unwrap();

    let node_url =
        std::env::var("CHEATNET_RPC_URL").expect("CHEATNET_RPC_URL must be set in the .env file");

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
            #[fork(url: "{}", block_id: BlockId::Tag(BlockTag::Latest))]
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
        node_url
    ).as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn fork_aliased_decorator() {
    dotenv().ok().unwrap();
    let node_url =
        std::env::var("CHEATNET_RPC_URL").expect("CHEATNET_RPC_URL must be set in the .env file");

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

    let result = run(
        &test.path().unwrap(),
        &String::from("src"),
        &test.path().unwrap().join("src/lib.cairo"),
        &Some(test.linked_libraries()),
        &RunnerConfig::new(
            None,
            false,
            false,
            &ForgeConfig {
                exit_first: false,
                fork: Some(vec![PredefinedFork {
                    name: "FORK_NAME_FROM_SCARB_TOML".to_string(),
                    url: node_url,
                    block_id: HashMap::from([("tag".to_string(), "Latest".to_string())]),
                }]),
            },
        ),
        &corelib_path(),
        &test.contracts(&corelib_path()).unwrap(),
        &Utf8PathBuf::from_path_buf(predeployed_contracts().to_path_buf()).unwrap(),
    )
    .unwrap();

    assert_passed!(result);
}
