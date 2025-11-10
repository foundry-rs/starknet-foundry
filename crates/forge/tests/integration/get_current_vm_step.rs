use crate::utils::runner::{Contract, assert_passed};
use crate::utils::running_tests::run_test_case;
use crate::utils::test_case;
use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use std::path::Path;

#[test]
fn test_get_current_vm_step() {
    let test = test_case!(
        indoc!(
            r#"
            use snforge_std::testing::get_current_vm_step;
            use snforge_std::{ContractClassTrait, DeclareResultTrait, declare};


            const STEPS_MARGIN: u32 = 250;

            // 1173 = cost of 1 deploy syscall without calldata
            const DEPLOY_SYSCALL_STEPS: u32 = 1173;

            // 903 = steps of 1 call contract syscall
            const CALL_CONTRACT_SYSCALL_STEPS: u32 = 903;

            // 90 = steps of 1 call contract syscall
            const STORAGE_READ_SYSCALL_STEPS: u32 = 90;

            const TOTAL_GET_BALANCE_CALL_STEPS: u32 = CALL_CONTRACT_SYSCALL_STEPS
                + STORAGE_READ_SYSCALL_STEPS
                + 29; // ~29 comes from VM

            #[test]
            fn check_current_vm_step() {
                let contract = declare("HelloStarknet").unwrap().contract_class();
                let step_a = get_current_vm_step();

                let (contract_address, _) = contract.deploy(@ArrayTrait::new()).unwrap();

                // Step after calling deploy syscall from (as top call)
                let step_b = get_current_vm_step();

                let expected_lower = DEPLOY_SYSCALL_STEPS + step_a - STEPS_MARGIN;
                let expected_upper = DEPLOY_SYSCALL_STEPS + step_a + STEPS_MARGIN;
                assert!(
                    expected_lower <= step_b && step_b <= expected_upper,
                    "step_b ({step_b}) not in [{expected_lower}, {expected_upper}]",
                );

                let dispatcher = IHelloStarknetDispatcher { contract_address };

                // call get_balance 5 times to accumulate some steps
                let _balance = dispatcher.get_balance();
                let _balance = dispatcher.get_balance();
                let _balance = dispatcher.get_balance();
                let _balance = dispatcher.get_balance();
                let _balance = dispatcher.get_balance();

                // Step after calling call_contract syscall (as top call) and
                // storage read syscall (as inner call)
                let step_c = get_current_vm_step();
                let expected_lower = step_b + (TOTAL_GET_BALANCE_CALL_STEPS) * 5 - STEPS_MARGIN;
                let expected_upper = step_b + (TOTAL_GET_BALANCE_CALL_STEPS) * 5 + STEPS_MARGIN;
                assert!(
                    expected_lower <= step_c && step_c <= expected_upper,
                    "step_c ({step_c}) not in [{expected_lower}, {expected_upper}]",
                );
            }
        "#
        ),
        Contract::from_code_path(
            "HelloStarknet".to_string(),
            Path::new("tests/data/contracts/hello_starknet.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);

    assert_passed(&result);
}
