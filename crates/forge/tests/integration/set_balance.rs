use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use std::path::Path;
use test_utils::runner::{Contract, assert_passed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn test_set_balance_strk() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::TokenTrait;
            use starknet::{ContractAddress, syscalls, SyscallResultTrait};

            use snforge_std::{set_balance, Token};

            fn get_balance(contract_address: ContractAddress, token: Token) -> Span<felt252> {
                let mut calldata: Array<felt252> = array![contract_address.into()];
                let balance = syscalls::call_contract_syscall(
                    token.contract_address(), selector!("balance_of"), calldata.span(),
                )
                    .unwrap_syscall();
                balance
            }

            #[test]
            fn test_set_balance_strk() {
                let contract_address: ContractAddress = 0x123.try_into().unwrap();

                let balance_before = get_balance(contract_address, Token::STRK);
                assert_eq!(balance_before, array![0, 0].span(), "Balance should be 0");

                set_balance(contract_address, 10, Token::STRK);

                let balance_after = get_balance(contract_address, Token::STRK);
                assert_eq!(balance_after, array![10, 0].span(), "Balance should be 10");
            }
        "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/simple_package/src/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn test_set_balance_custom_token() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::TokenTrait;
            use starknet::{ContractAddress, syscalls, SyscallResultTrait};

            use snforge_std::{declare, ContractClassTrait, DeclareResultTrait, set_balance, Token, CustomToken};

            fn deploy_contract(
                name: ByteArray, constructor_calldata: Option<Array<felt252>>,
            ) -> ContractAddress {
                let contract = declare(name).unwrap().contract_class();
                let (contract_address, _) = contract.deploy(@constructor_calldata.unwrap_or(array![])).unwrap();
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
                let token_address = deploy_contract("ERC20", Option::Some(constructor_calldata));
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
        Contract::from_code_path(
            "ERC20".to_string(),
            Path::new("tests/data/contracts/erc20.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn test_set_balance_big_amount() {
    let test = test_case!(
        indoc!(
            r#"
            use core::num::traits::Pow;
            use snforge_std::TokenTrait;
            use starknet::{ContractAddress, syscalls, SyscallResultTrait};

            use snforge_std::{set_balance, Token};

            fn get_balance(contract_address: ContractAddress, token: Token) -> Span<felt252> {
                let mut calldata: Array<felt252> = array![contract_address.into()];
                let balance = syscalls::call_contract_syscall(
                    token.contract_address(), selector!("balance_of"), calldata.span(),
                )
                    .unwrap_syscall();
                balance
            }

            #[test]
            fn test_set_balance_big_amount() {
                let contract_address: ContractAddress = 0x123.try_into().unwrap();

                let balance_before = get_balance(contract_address, Token::STRK);
                assert_eq!(balance_before, array![0, 0].span(), "Balance should be 0");

                set_balance(contract_address, (10.pow(50_u32)).try_into().unwrap(), Token::STRK);

                let balance_after = get_balance(contract_address, Token::STRK);
                assert_eq!(
                    balance_after,
                    array![194599656488044247630319707454198251520, 293873587705].span(),
                    "Balance should should be 10^50",
                );
            }
        "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/simple_package/src/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}

#[test]
fn test_set_balance_strk_with_fork() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::TokenTrait;
            use starknet::{ContractAddress, syscalls, SyscallResultTrait};

            use snforge_std::{set_balance, Token};

            fn get_balance(contract_address: ContractAddress, token: Token) -> Span<felt252> {
                let mut calldata: Array<felt252> = array![contract_address.into()];
                let balance = syscalls::call_contract_syscall(
                    token.contract_address(), selector!("balance_of"), calldata.span(),
                )
                    .unwrap_syscall();
                balance
            }

            #[fork(url: "http://188.34.188.184:7070/rpc/v0_8", block_number: 715_593)]
            #[test]
            fn test_set_balance_strk_with_fork() {
                let contract_address: ContractAddress =
                    0x0585dd8cab667ca8415fac8bead99c78947079aa72d9120140549a6f2edc4128
                    .try_into()
                    .unwrap();

                let balance_before = get_balance(contract_address, Token::STRK);
                assert_eq!(balance_before, array![109394843313476728397, 0].span());

                set_balance(contract_address, 10, Token::STRK);

                let balance_after = get_balance(contract_address, Token::STRK);
                assert_eq!(balance_after, array![10, 0].span(), "Balance should should be 10");
            }
        "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/simple_package/src/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}
