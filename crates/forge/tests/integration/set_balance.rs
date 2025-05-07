use cheatnet::predeployment::eth::ETH_CONTRACT_ADDRESS;
use cheatnet::predeployment::strk::STRK_CONTRACT_ADDRESS;
use forge_runner::forge_config::ForgeTrackedResource;
use indoc::{formatdoc, indoc};
use shared::test_utils::node_url::node_rpc_url;
use std::path::Path;
use test_case::test_case;
use test_utils::runner::{Contract, assert_case_output_contains, assert_failed, assert_passed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case as util_test_case;

#[test_case("STRK";"strk")]
#[test_case("ETH";"eth")]
fn test_set_balance_predefined_token(token: &str) {
    let test = util_test_case!(
        formatdoc!(
            r#"
            use snforge_std::{{set_balance, Token, TokenTrait}};
            use starknet::{{ContractAddress, syscalls, SyscallResultTrait}};

            fn get_balance(contract_address: ContractAddress, token: Token) -> Span<felt252> {{
                let mut calldata: Array<felt252> = array![contract_address.into()];
                let balance = syscalls::call_contract_syscall(
                    token.contract_address(), selector!("balance_of"), calldata.span(),
                )
                    .unwrap_syscall();
                balance
            }}

            #[test]
            fn test_set_balance_predefined_token() {{
                let contract_address: ContractAddress = 0x123.try_into().unwrap();

                let balance_before = get_balance(contract_address, Token::{});
                assert_eq!(balance_before, array![0, 0].span(), "Balance should be 0");

                set_balance(contract_address, 10, Token::{});

                let balance_after = get_balance(contract_address, Token::{});
                assert_eq!(balance_after, array![10, 0].span(), "Balance should be 10");
        }}
        "#,
            token,
            token,
            token,
        )
        .as_str(),
        Contract::from_code_path(
            "HelloStarknet",
            Path::new("tests/data/simple_package/src/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn test_set_balance_custom_token() {
    let test = util_test_case!(
        indoc!(
            r#"
            use snforge_std::{declare, set_balance, Token, TokenTrait, CustomToken, ContractClassTrait, DeclareResultTrait,};
            use starknet::{ContractAddress, syscalls, SyscallResultTrait};

            fn deploy_contract(
                name: ByteArray, constructor_calldata: Array<felt252>,
            ) -> ContractAddress {
                let contract = declare(name).unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@constructor_calldata).unwrap();
                contract_address
            }

            fn get_balance(contract_address: ContractAddress, token: Token) -> Span<felt252> {
                let mut calldata: Array<felt252> = array![contract_address.into()];
                let balance = syscalls::call_contract_syscall(
                    token.contract_address(), selector!("balance_of"), calldata.span(),
                )
                    .unwrap_syscall();
                balance
            }

            #[test]
            fn test_set_balance_custom_token() {
                let contract_address: ContractAddress = 0x123.try_into().unwrap();

                let constructor_calldata: Array<felt252> = array![
                    'CustomToken'.into(), 'CT'.into(), 18.into(), 1_000_000_000.into(), 0.into(), 123.into(),
                ];
                let token_address = deploy_contract("ERC20", constructor_calldata);
                let custom_token = Token::Custom(
                    CustomToken {
                        contract_address: token_address, balances_variable_selector: selector!("balances"),
                    },
                );

                let balance_before = get_balance(contract_address, custom_token);
                assert_eq!(balance_before, array![0, 0].span(), "Balance should be 0");

                set_balance(contract_address, 10, custom_token);

                let balance_after = get_balance(contract_address, custom_token);
                assert_eq!(balance_after, array![10, 0].span(), "Balance should be 10");
            }
        "#
        ),
        Contract::from_code_path("ERC20", Path::new("tests/data/contracts/erc20.cairo"),).unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test_case("STRK";"strk")]
#[test_case("ETH";"eth")]
fn test_set_balance_big_amount(token: &str) {
    let test = util_test_case!(
        format!(
            r#"
            use core::num::traits::Pow;
            use snforge_std::{{set_balance, Token, TokenTrait}};
            use starknet::{{ContractAddress, syscalls, SyscallResultTrait}};

            fn get_balance(contract_address: ContractAddress, token: Token) -> Span<felt252> {{
                let mut calldata: Array<felt252> = array![contract_address.into()];
                let balance = syscalls::call_contract_syscall(
                    token.contract_address(), selector!("balance_of"), calldata.span(),
                )
                    .unwrap_syscall();
                balance
            }}

            #[test]
            fn test_set_balance_big_amount() {{
                let contract_address: ContractAddress = 0x123.try_into().unwrap();

                let balance_before = get_balance(contract_address, Token::{token});
                assert_eq!(balance_before, array![0, 0].span(), "Balance should be 0");

                set_balance(contract_address, (10.pow(50_u32)).try_into().unwrap(), Token::{token});

                let balance_after = get_balance(contract_address, Token::{token});
                assert_eq!(
                    balance_after,
                    array![194599656488044247630319707454198251520, 293873587705].span(),
                    "Balance should should be 10^50",
                );
            }}
        "#
        )
        .as_str(),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/simple_package/src/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test_case("STRK", [109_394_843_313_476_728_397_u128, 0];"strk")]
#[test_case("ETH", [24_969_862_322_663_205, 0];"eth")]
fn test_set_balance_with_fork(token: &str, balance_before: [u128; 2]) {
    let balance_before_low = balance_before[0];
    let balance_before_high = balance_before[1];
    let test = util_test_case!(
        formatdoc!(
            r#"
            use snforge_std::{{set_balance, Token, TokenTrait}};
            use starknet::{{ContractAddress, syscalls, SyscallResultTrait}};

            fn get_balance(contract_address: ContractAddress, token: Token) -> Span<felt252> {{
                let mut calldata: Array<felt252> = array![contract_address.into()];
                let balance = syscalls::call_contract_syscall(
                    token.contract_address(), selector!("balance_of"), calldata.span(),
                )
                    .unwrap_syscall();
                balance
        }}

            #[fork(url: "{}", block_number: 715_593)]
            #[test]
            fn test_set_balance_strk_with_fork() {{
                let contract_address: ContractAddress =
                    0x0585dd8cab667ca8415fac8bead99c78947079aa72d9120140549a6f2edc4128
                    .try_into()
                    .unwrap();

                let balance_before = get_balance(contract_address, Token::{token});
                assert_eq!(balance_before, array![{balance_before_low}, {balance_before_high}].span());

                set_balance(contract_address, 10, Token::{token});

                let balance_after = get_balance(contract_address, Token::{token});
                assert_eq!(balance_after, array![10, 0].span(), "Balance should should be 10");
        }}
        "#,
            node_rpc_url(),
        )
        .as_str(),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/simple_package/src/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test_case("STRK", STRK_CONTRACT_ADDRESS; "strk")]
#[test_case("ETH", ETH_CONTRACT_ADDRESS; "eth")]
fn test_set_balance_with_disabled_predeployment(token: &str, contract_address: &str) {
    let test = util_test_case!(
        formatdoc!(
            r#"
            use snforge_std::{{Token, TokenTrait}};
            use starknet::{{ContractAddress, syscalls, SyscallResultTrait}};

            fn get_balance(contract_address: ContractAddress, token: Token) -> Span<felt252> {{
                let mut calldata: Array<felt252> = array![contract_address.into()];
                let balance = syscalls::call_contract_syscall(
                    token.contract_address(), selector!("balance_of"), calldata.span(),
                )
                    .unwrap_syscall();
                balance
            }}

            #[test]
            #[disable_predeployed_contracts]
            fn test_set_balance_strk_with_disabled_predeployment() {{
                let contract_address: ContractAddress = 0x123.try_into().unwrap();
                get_balance(contract_address, Token::{});
            }}
        "#,
            token
        )
        .as_str(),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/simple_package/src/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_failed(&result);

    let asserted_msg = format!("Contract not deployed at address: {contract_address}");
    assert_case_output_contains(
        &result,
        "test_set_balance_strk_with_disabled_predeployment",
        &asserted_msg,
    );
}
