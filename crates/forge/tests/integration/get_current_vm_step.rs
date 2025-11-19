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


            const STEPS_MARGIN: u32 = 100;

            // 1173 = cost of 1 deploy syscall without calldata
            const DEPLOY_SYSCALL_STEPS: u32 = 1173;

            // 903 = steps of 1 call contract syscall
            const CALL_CONTRACT_SYSCALL_STEPS: u32 = 903;

            // 90 = steps of 1 call contract syscall
            const STORAGE_READ_SYSCALL_STEPS: u32 = 90;

            #[test]
            fn check_current_vm_step() {
                let contract = declare("HelloStarknet").unwrap().contract_class();
                let step_a = get_current_vm_step();

                let (contract_address_a, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                let (contract_address_b, _) = contract.deploy(@ArrayTrait::new()).unwrap();
                // Sycalls between step_a and step_b:
                // top call: 2 x deploy syscall
                // inner call: -/-
                let step_b = get_current_vm_step();

                let expected_steps_taken = 2 * DEPLOY_SYSCALL_STEPS + 130; // 130 are steps from VM
                let expected_lower = expected_steps_taken + step_a - STEPS_MARGIN;
                let expected_upper = expected_steps_taken + step_a + STEPS_MARGIN;
                assert!(
                    expected_lower <= step_b && step_b <= expected_upper,
                    "step_b ({step_b}) not in [{expected_lower}, {expected_upper}]",
                );

                let dispatcher_a = IHelloStarknetDispatcher { contract_address: contract_address_a };

                // contract A calls `get_balance` from contract B
                let _balance = dispatcher_a
                    .call_other_contract(
                        contract_address_b.try_into().unwrap(), selector!("get_balance"), None,
                    );

                // Sycalls between step_b and step_c:
                // top call: 1 x call contract syscall
                // inner calls: 1 x storage read syscall, 1 x call contract syscall
                let step_c = get_current_vm_step();

                let expected_steps_taken = 2 * CALL_CONTRACT_SYSCALL_STEPS
                    + 1 * STORAGE_READ_SYSCALL_STEPS
                    + 277; // 277 are steps from VM
                let expected_lower = expected_steps_taken + step_b - STEPS_MARGIN;
                let expected_upper = expected_steps_taken + step_b + STEPS_MARGIN;
                assert!(
                    expected_lower <= step_c && step_c <= expected_upper,
                    "step_c ({step_c}) not in [{expected_lower}, {expected_upper}]",
                );
            }

            #[starknet::interface]
            pub trait IHelloStarknet<TContractState> {
                fn get_balance(self: @TContractState) -> felt252;
                fn call_other_contract(
                    self: @TContractState,
                    other_contract_address: felt252,
                    selector: felt252,
                    calldata: Option<Array<felt252>>,
                ) -> Span<felt252>;
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
