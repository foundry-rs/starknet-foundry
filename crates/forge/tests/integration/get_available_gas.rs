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

                // call_contract syscall: 91_560 gas (because 1 * 903 * 100 + 18 * 70)
                //      -> 1 call contract syscall costs 903 cairo steps and 18 range check builtins
                //      -> CallContract os_resources:
                //      (blockifier 0.19.0-rc.2) https://github.com/starkware-libs/sequencer/blob/773c57afc7c450a1122a57c914b10f74df2492ea/crates/blockifier/resources/blockifier_versioned_constants_0_14_3.json#L229-L234
                // storage_write syscall: 59_970 gas (because 1 * 599 * 100 + 1 * 70)
                //      -> 1 storage write syscall costs 599 cairo steps and 1 range check builtin
                //      -> StorageWrite os_resources:
                //      (blockifier 0.19.0-rc.2) https://github.com/starkware-libs/sequencer/blob/773c57afc7c450a1122a57c914b10f74df2492ea/crates/blockifier/resources/blockifier_versioned_constants_0_14_3.json#L488-L494
                // storage_read syscall: 24_070 gas (because 1 * 240 * 100 + 1 * 70)
                //      -> 1 storage read syscall costs 240 cairo steps and 1 range check builtin
                //      -> StorageRead os_resources:
                //      (blockifier 0.19.0-rc.2) https://github.com/starkware-libs/sequencer/blob/773c57afc7c450a1122a57c914b10f74df2492ea/crates/blockifier/resources/blockifier_versioned_constants_0_14_3.json#L481-L486
                let min_expected_gas = 91_560 + 59_970 + 24_070;

                // Check that gas used is above the expected minimum
                assert(gas_diff > min_expected_gas, 'Incorrect gas');

                // Allow an arbitrary margin of 10_000 gas
                assert(min_expected_gas + 10_000 > gas_diff, 'Incorrect gas');
            }
        "#
        ),
        Contract::from_code_path(
            "contract::HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::SierraGas);

    assert_passed(&result);
}
