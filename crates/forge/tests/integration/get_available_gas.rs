use crate::utils::runner::{Contract, assert_passed};
use crate::utils::running_tests::run_test_case;
use crate::utils::test_case;
use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use std::path::Path;

#[test]
fn test_get_available_gas() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::{declare, DeclareResultTrait, ContractClassTrait};

            #[test]
            fn check_available_gas() {
                let contract = declare("HelloStarknet").unwrap().contract_class().clone();
                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                let gas_before = core::testing::get_available_gas();
                core::gas::withdraw_gas().unwrap();

                starknet::syscalls::call_contract_syscall(
                    contract_address, selector!("increase_balance"), array![10].span(),
                ).unwrap();

                let gas_after = core::testing::get_available_gas();
                core::gas::withdraw_gas().unwrap();

                let gas_diff = gas_before - gas_after;

                // call_contract syscall: 91_560 gas
                // storage_write and storage_read syscalls: 10_000 gas each
                let min_expected_gas = 91_560 + 10_000 * 2;

                // Check that gas used is above the expected minimum
                assert(gas_diff > min_expected_gas, 'Incorrect gas');

                // Allow an arbitrary margin of 10_000 gas
                assert(min_expected_gas + 10_000 > gas_diff, 'Incorrect gas');
            }
        "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_passed(&result);
}
