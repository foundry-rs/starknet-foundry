use indoc::formatdoc;
use test_utils::running_tests::run_test_case;
use test_utils::{assert_passed, test_case};

static CHEATNET_RPC_URL: &str = "http://188.34.188.184:9545/rpc/v0_6";

#[test]
fn prank_cairo0_contract() {
    let test = test_case!(formatdoc!(
        r#"
            use starknet::{{class_hash::Felt252TryIntoClassHash, SyscallResultTrait}};
            use snforge_std::{{start_prank, stop_prank, test_address, CheatTarget}};

            const CAIRO0_CLASS_HASH: felt252 = 3390338629460413397996224645413818793848470654644268493965292562067946505747;
            const LIB_CALL_SELECTOR: felt252 = 219972792400094465318120350250971259539342451068659710037080072200128459645;

            #[test]
            #[fork(url: "{}", block_id: BlockId::Number(314821))]
            fn prank_cairo0_contract() {{
                let caller = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                start_prank(CheatTarget::One(test_address()), 123.try_into().unwrap());

                let pranked_caller = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                stop_prank(CheatTarget::One(test_address()));

                let unpranked_caller = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                assert(*pranked_caller == 123, 'start_prank does not work');
                assert(unpranked_caller == caller, 'stop_prank does not work');
            }}
        "#,
        CHEATNET_RPC_URL,
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn roll_cairo0_contract() {
    let test = test_case!(formatdoc!(
        r#"
            use starknet::{{class_hash::Felt252TryIntoClassHash, SyscallResultTrait}};
            use snforge_std::{{start_roll, stop_roll, test_address, CheatTarget}};

            const CAIRO0_CLASS_HASH: felt252 = 3390338629460413397996224645413818793848470654644268493965292562067946505747;
            const LIB_CALL_SELECTOR: felt252 = 1043360521069001059812816533306435120284814797591254795559962622467917544215;

            #[test]
            #[fork(url: "{}", block_id: BlockId::Number(314821))]
            fn roll_cairo0_contract() {{
                let block_number = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                start_roll(CheatTarget::One(test_address()), 123);

                let rolled_block_number = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                stop_roll(CheatTarget::One(test_address()));

                let unrolled_block_number = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                assert(*rolled_block_number == 123, 'start_roll does not work');
                assert(unrolled_block_number == block_number, 'stop_roll does not work');
            }}
        "#,
        CHEATNET_RPC_URL,
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn warp_cairo0_contract() {
    let test = test_case!(formatdoc!(
        r#"
            use starknet::{{class_hash::Felt252TryIntoClassHash, SyscallResultTrait}};
            use snforge_std::{{start_warp, stop_warp, test_address, CheatTarget}};

            const CAIRO0_CLASS_HASH: felt252 = 3390338629460413397996224645413818793848470654644268493965292562067946505747;
            const LIB_CALL_SELECTOR: felt252 = 1104673410415683966349700971986586038248888383055081852378797598061780438342;

            #[test]
            #[fork(url: "{}", block_id: BlockId::Number(314821))]
            fn warp_cairo0_contract() {{
                let block_timestamp = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                start_warp(
                    CheatTarget::One(test_address()), 123
                );

                let warped_block_timestamp = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                stop_warp(CheatTarget::One(test_address()));

                let unwarped_block_timestamp = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                assert(*warped_block_timestamp == 123, 'start_warp does not work');
                assert(unwarped_block_timestamp == block_timestamp, 'stop_warp does not work');
            }}
        "#,
        CHEATNET_RPC_URL,
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn mock_call_cairo0_contract() {
    let test = test_case!(formatdoc!(
        r#"
            use starknet::{{contract_address_const}};
            use snforge_std::{{start_mock_call, stop_mock_call}};

            #[starknet::interface]
            trait IERC20<TContractState> {{
                fn name(self: @TContractState) -> felt252;
            }}

            #[test]
            #[fork(url: "{}", block_id: BlockId::Number(314821))]
            fn mock_call_cairo0_contract() {{
                let eth_dispatcher = IERC20Dispatcher {{
                    contract_address: contract_address_const::<
                        0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7
                    >()
                }};

                assert(eth_dispatcher.name() == 'ETH', 'invalid name');

                start_mock_call(eth_dispatcher.contract_address, 'name', 'NotETH');

                assert(eth_dispatcher.name() == 'NotETH', 'invalid mocked name');

                stop_mock_call(eth_dispatcher.contract_address, 'name');

                assert(eth_dispatcher.name() == 'ETH', 'invalid name after mock');
            }}
        "#,
        CHEATNET_RPC_URL,
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
}

#[test]
fn store_load_cairo0_contract() {
    let test = test_case!(formatdoc!(
        r#"
            use starknet::{{contract_address_const}};
            use snforge_std::{{store, load}};

            #[starknet::interface]
            trait IERC20<TContractState> {{
                fn name(self: @TContractState) -> felt252;
            }}

            #[test]
            #[fork(url: "{}", block_id: BlockId::Number(314821))]
            fn mock_call_cairo0_contract() {{
                let eth_dispatcher = IERC20Dispatcher {{
                    contract_address: contract_address_const::<
                        0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7
                    >()
                }};

                assert(eth_dispatcher.name() == 'ETH', 'invalid name');

                let name = load(eth_dispatcher.contract_address, selector!("ERC20_name"), 1);

                assert(name == array!['ETH'], 'invalid load value');

                store(eth_dispatcher.contract_address, selector!("ERC20_name"), array!['NotETH'].span());

                assert(eth_dispatcher.name() == 'NotETH', 'invalid store name');
                
                let name = load(eth_dispatcher.contract_address, selector!("ERC20_name"), 1);
                
                assert(name == array!['NotETH'], 'invalid load2 name');
            }}
        "#,
        CHEATNET_RPC_URL,
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed!(result);
}
