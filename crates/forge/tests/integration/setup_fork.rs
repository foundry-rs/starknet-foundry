use crate::integration::common::running_tests::run_test_case;
use crate::{assert_passed, test_case};
use dotenv::dotenv;
use indoc::formatdoc;

#[test]
fn fork_simple_cheatcode() {
    dotenv().ok();
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
            use snforge_std::{{ ForkConfig, ForkTrait, BlockTag, BlockId }};

            #[starknet::interface]
            trait IHelloStarknet<TContractState> {{
                fn increase_balance(ref self: TContractState, amount: felt252);
                fn get_balance(self: @TContractState) -> felt252;
            }}

            #[test]
            fn test_fork_simple() {{
                let fork_config = ForkConfig {{
                    url: array!['{}', '{}'],
                    block: BlockId::Tag(BlockTag::Latest(()))
                }};
                fork_config.set_up();

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
        node_url[..31].to_string(),
        node_url[31..].to_string()
    ).as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn fork_simple_decorator() {
    dotenv().ok();
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
            use snforge_std::{{ ForkConfig, ForkTrait, BlockTag, BlockId }};

            #[starknet::interface]
            trait IHelloStarknet<TContractState> {{
                fn increase_balance(ref self: TContractState, amount: felt252);
                fn get_balance(self: @TContractState) -> felt252;
            }}

            #[test]
            #[fork(url: "{}", block_id: BlockId::Tag(BlockTag::Latest(())))]
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
