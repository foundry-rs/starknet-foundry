use indoc::formatdoc;
use shared::test_utils::node_url::node_rpc_url;
use test_utils::runner::assert_passed;
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn cheat_caller_address_cairo0_contract() {
    let test = test_case!(formatdoc!(
        r#"
            use starknet::{{class_hash::Felt252TryIntoClassHash, SyscallResultTrait}};
            use snforge_std::{{start_cheat_caller_address, stop_cheat_caller_address, test_address}};

            const CAIRO0_CLASS_HASH: felt252 = 0x029c0caff0aef71bd089d58b25bcc5c23458d080b2d1b75e423de86f95176818;
            const LIB_CALL_SELECTOR: felt252 = 219972792400094465318120350250971259539342451068659710037080072200128459645;

            #[test]
            #[fork(url: "{}", block_number: 54060)]
            fn cheat_caller_address_cairo0_contract() {{
                let caller = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                start_cheat_caller_address(test_address(), 123.try_into().unwrap());

                let cheated_caller_address = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                stop_cheat_caller_address(test_address());

                let uncheated_caller_address = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                assert(*cheated_caller_address == 123, 'does not work');
                assert(uncheated_caller_address == caller, 'does not work');
            }}
        "#,
        node_rpc_url(),
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn cheat_block_number_cairo0_contract() {
    let test = test_case!(formatdoc!(
        r#"
            use starknet::{{class_hash::Felt252TryIntoClassHash, SyscallResultTrait}};
            use snforge_std::{{start_cheat_block_number, stop_cheat_block_number, test_address}};

            const CAIRO0_CLASS_HASH: felt252 = 0x029c0caff0aef71bd089d58b25bcc5c23458d080b2d1b75e423de86f95176818;
            const LIB_CALL_SELECTOR: felt252 = 1043360521069001059812816533306435120284814797591254795559962622467917544215;

            #[test]
            #[fork(url: "{}", block_number: 54060)]
            fn cheat_block_number_cairo0_contract() {{
                let block_number = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                start_cheat_block_number(test_address(), 123);

                let cheated_block_number = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                stop_cheat_block_number(test_address());

                let uncheated_block_number = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                assert(*cheated_block_number == 123, 'does not work');
                assert(uncheated_block_number == block_number, 'does not work');
            }}
        "#,
        node_rpc_url(),
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn cheat_block_timestamp_cairo0_contract() {
    let test = test_case!(formatdoc!(
        r#"
            use starknet::{{class_hash::Felt252TryIntoClassHash, SyscallResultTrait}};
            use snforge_std::{{start_cheat_block_timestamp, stop_cheat_block_timestamp, test_address}};

            const CAIRO0_CLASS_HASH: felt252 = 0x029c0caff0aef71bd089d58b25bcc5c23458d080b2d1b75e423de86f95176818;
            const LIB_CALL_SELECTOR: felt252 = 1104673410415683966349700971986586038248888383055081852378797598061780438342;

            #[test]
            #[fork(url: "{}", block_number: 54060)]
            fn cheat_block_timestamp_cairo0_contract() {{
                let block_timestamp = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                start_cheat_block_timestamp(
                    test_address(), 123
                );

                let cheated_block_timestamp = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                stop_cheat_block_timestamp(test_address());

                let uncheated_block_timestamp = starknet::library_call_syscall(
                    CAIRO0_CLASS_HASH.try_into().unwrap(),
                    LIB_CALL_SELECTOR,
                    array![].span(),
                ).unwrap_syscall()[0];

                assert(*cheated_block_timestamp == 123, 'does not work');
                assert(uncheated_block_timestamp == block_timestamp, 'does not work');
            }}
        "#,
        node_rpc_url(),
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed(&result);
}

#[test]
fn mock_call_cairo0_contract() {
    let test = test_case!(
        formatdoc!(
            r#"
            use starknet::{{contract_address_const}};
            use snforge_std::{{start_mock_call, stop_mock_call}};

            #[starknet::interface]
            trait IERC20<TContractState> {{
                fn name(self: @TContractState) -> felt252;
            }}

            #[test]
            #[fork(url: "{}", block_number: 54060)]
            fn mock_call_cairo0_contract() {{
                let eth_dispatcher = IERC20Dispatcher {{
                    contract_address: contract_address_const::<
                        0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7
                    >()
                }};

                assert(eth_dispatcher.name() == 'Ether', 'invalid name');

                start_mock_call(eth_dispatcher.contract_address, selector!("name"), 'NotEther');

                assert(eth_dispatcher.name() == 'NotEther', 'invalid mocked name');

                stop_mock_call(eth_dispatcher.contract_address, selector!("name"));

                assert(eth_dispatcher.name() == 'Ether', 'invalid name after mock');
            }}
        "#,
            node_rpc_url(),
        )
        .as_str()
    );

    let result = run_test_case(&test);

    assert_passed(&result);
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
            #[fork(url: "{}", block_number: 54060)]
            fn mock_call_cairo0_contract() {{
                let eth_dispatcher = IERC20Dispatcher {{
                    contract_address: contract_address_const::<
                        0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7
                    >()
                }};

                assert(eth_dispatcher.name() == 'Ether', 'invalid name');

                let name = load(eth_dispatcher.contract_address, selector!("ERC20_name"), 1);

                assert(name == array!['Ether'], 'invalid load value');

                store(eth_dispatcher.contract_address, selector!("ERC20_name"), array!['NotEther'].span());

                assert(eth_dispatcher.name() == 'NotEther', 'invalid store name');
                
                let name = load(eth_dispatcher.contract_address, selector!("ERC20_name"), 1);
                
                assert(name == array!['NotEther'], 'invalid load2 name');
            }}
        "#,
        node_rpc_url(),
    )
    .as_str());

    let result = run_test_case(&test);

    assert_passed(&result);
}
