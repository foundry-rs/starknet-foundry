use crate::integration::common::running_tests::run_test_case;
use crate::{assert_passed, test_case};
use indoc::formatdoc;

const CAIRO0_TESTER_ADDRESS: &str =
    "1825832089891106126806210124294467331434544162488231781791271899226056323189";
static CHEATNET_RPC_URL: &str = "http://188.34.188.184:9545/rpc/v0.4";

#[test]
fn prank_cairo0_contract() {
    let test = test_case!(formatdoc!(
        r#"
            use starknet::contract_address_const;
            use snforge_std::{{start_prank, stop_prank}};
            use debug::PrintTrait;

            #[starknet::interface]
            trait Cairo0Tester<TState> {{
                fn return_caller_address(self: @TState) -> felt252;
            }}

            #[test]
            #[fork(url: "{}", block_id: BlockId::Number(314821))]
            fn test() {{
                let contract_address = contract_address_const::<{}>();

                let dispatcher = Cairo0TesterDispatcher {{ contract_address }};

                let caller = dispatcher.return_caller_address();
                start_prank(contract_address, 123.try_into().unwrap());

                let pranked_caller = dispatcher.return_caller_address();

                stop_prank(contract_address);
                let unpranked_caller = dispatcher.return_caller_address();

                assert(pranked_caller == 123, 'start_prank does not work');
                assert(unpranked_caller == caller, 'stop_prank does not work');
            }}
        "#,
        CHEATNET_RPC_URL,
        CAIRO0_TESTER_ADDRESS,
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn prank_proxied_cairo0_contract() {
    let test = test_case!(formatdoc!(
        r#"
            use starknet::contract_address_const;
            use snforge_std::{{start_prank, stop_prank}};
            use debug::PrintTrait;

            #[starknet::interface]
            trait Cairo0Tester<TState> {{
                fn return_proxied_caller_address(self: @TState) -> felt252;
            }}

            #[test]
            #[fork(url: "{}", block_id: BlockId::Number(314821))]
            fn test() {{
                let contract_address = contract_address_const::<{}>();

                let dispatcher = Cairo0TesterDispatcher {{ contract_address }};

                let caller = dispatcher.return_proxied_caller_address();
                start_prank(contract_address, 123.try_into().unwrap());

                let pranked_caller = dispatcher.return_proxied_caller_address();

                stop_prank(contract_address);
                let unpranked_caller = dispatcher.return_proxied_caller_address();

                assert(pranked_caller == 123, 'start_prank does not work');
                assert(unpranked_caller == caller, 'stop_prank does not work');
            }}
        "#,
        CHEATNET_RPC_URL,
        CAIRO0_TESTER_ADDRESS,
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn roll_cairo0_contract() {
    let test = test_case!(formatdoc!(
        r#"
            use starknet::contract_address_const;
            use snforge_std::{{start_roll, stop_roll}};
            use debug::PrintTrait;

            #[starknet::interface]
            trait Cairo0Tester<TState> {{
                fn return_block_number(self: @TState) -> felt252;
            }}

            #[test]
            #[fork(url: "{}", block_id: BlockId::Number(314821))]
            fn test() {{
                let contract_address = contract_address_const::<{}>();

                let dispatcher = Cairo0TesterDispatcher {{ contract_address }};

                let block_number = dispatcher.return_block_number();
                start_roll(contract_address, 123);

                let rolled_block_number = dispatcher.return_block_number();

                stop_roll(contract_address);
                let unrolled_block_number = dispatcher.return_block_number();

                assert(rolled_block_number == 123, 'start_roll does not work');
                assert(unrolled_block_number == block_number, 'stop_roll does not work');
            }}
        "#,
        CHEATNET_RPC_URL,
        CAIRO0_TESTER_ADDRESS,
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn roll_proxied_cairo0_contract() {
    let test = test_case!(formatdoc!(
        r#"
            use starknet::contract_address_const;
            use snforge_std::{{start_roll, stop_roll}};
            use debug::PrintTrait;

            #[starknet::interface]
            trait Cairo0Tester<TState> {{
                fn return_proxied_block_number(self: @TState) -> felt252;
            }}

            #[test]
            #[fork(url: "{}", block_id: BlockId::Number(314821))]
            fn test() {{
                let contract_address = contract_address_const::<{}>();

                let dispatcher = Cairo0TesterDispatcher {{ contract_address }};

                let block_number = dispatcher.return_proxied_block_number();
                start_roll(contract_address, 123);

                let rolled_block_number = dispatcher.return_proxied_block_number();

                stop_roll(contract_address);
                let unrolled_block_number = dispatcher.return_proxied_block_number();

                assert(rolled_block_number == 123, 'start_roll does not work');
                assert(unrolled_block_number == block_number, 'stop_roll does not work');
            }}
        "#,
        CHEATNET_RPC_URL,
        CAIRO0_TESTER_ADDRESS,
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn warp_cairo0_contract() {
    let test = test_case!(formatdoc!(
        r#"
            use starknet::contract_address_const;
            use snforge_std::{{start_warp, stop_warp}};
            use debug::PrintTrait;

            #[starknet::interface]
            trait Cairo0Tester<TState> {{
                fn return_block_timestamp(self: @TState) -> felt252;
            }}

            #[test]
            #[fork(url: "{}", block_id: BlockId::Number(314821))]
            fn test() {{
                let contract_address = contract_address_const::<{}>();

                let dispatcher = Cairo0TesterDispatcher {{ contract_address }};

                let block_timestamp = dispatcher.return_block_timestamp();
                start_warp(contract_address, 123);

                let warped_block_timestamp = dispatcher.return_block_timestamp();

                stop_warp(contract_address);
                let unwarped_block_timestamp = dispatcher.return_block_timestamp();

                assert(warped_block_timestamp == 123, 'start_warp does not work');
                assert(unwarped_block_timestamp == block_timestamp, 'stop_warp does not work');
            }}
        "#,
        CHEATNET_RPC_URL,
        CAIRO0_TESTER_ADDRESS,
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn warp_proxied_cairo0_contract() {
    let test = test_case!(formatdoc!(
        r#"
            use starknet::contract_address_const;
            use snforge_std::{{start_warp, stop_warp}};
            use debug::PrintTrait;

            #[starknet::interface]
            trait Cairo0Tester<TState> {{
                fn return_proxied_block_timestamp(self: @TState) -> felt252;
            }}

            #[test]
            #[fork(url: "{}", block_id: BlockId::Number(314821))]
            fn test() {{
                let contract_address = contract_address_const::<{}>();

                let dispatcher = Cairo0TesterDispatcher {{ contract_address }};

                let block_timestamp = dispatcher.return_proxied_block_timestamp();
                start_warp(contract_address, 123);

                let warped_block_timestamp = dispatcher.return_proxied_block_timestamp();

                stop_warp(contract_address);
                let unwarped_block_timestamp = dispatcher.return_proxied_block_timestamp();

                assert(warped_block_timestamp == 123, 'start_warp does not work');
                assert(unwarped_block_timestamp == block_timestamp, 'stop_warp does not work');
            }}
        "#,
        CHEATNET_RPC_URL,
        CAIRO0_TESTER_ADDRESS,
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
}
