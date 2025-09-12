use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use scarb_api::ScarbCommand;
use semver::Version;
use std::path::Path;
use test_utils::runner::{Contract, assert_passed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

// TODO(#3704) Remove scarb version check
fn skip_scarb_lt_2_11_0() -> bool {
    let version_info = ScarbCommand::version()
        .run()
        .expect("Failed to get Scarb version");

    if version_info.scarb < Version::new(2, 11, 0) {
        eprintln!("[IGNORED] `meta_tx_v0` syscall is not supported in Scarb < 2.11.0");
        true
    } else {
        false
    }
}

#[test]
fn meta_tx_v0_with_cheat_caller_address() {
    // TODO(#3704) Remove scarb version check
    if skip_scarb_lt_2_11_0() {
        return;
    }

    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use starknet::Felt252TryIntoContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, start_cheat_caller_address,
                stop_cheat_caller_address
            };

            #[starknet::interface]
            trait IMetaTxV0Test<TContractState> {
                fn execute_meta_tx_v0(
                    ref self: TContractState,
                    target: starknet::ContractAddress,
                    signature: Span<felt252>,
                ) -> felt252;
            }

            #[test]
            fn test_meta_tx_v0_with_cheat_caller_address() {
                let checker_contract = declare("CheatCallerAddressCheckerMetaTxV0").unwrap().contract_class();
                let (checker_address, _) = checker_contract.deploy(@ArrayTrait::new()).unwrap();

                let meta_contract = declare("MetaTxV0Test").unwrap().contract_class();
                let (meta_address, _) = meta_contract.deploy(@ArrayTrait::new()).unwrap();
                let meta_dispatcher = IMetaTxV0TestDispatcher { contract_address: meta_address };

                let signature = ArrayTrait::new();

                let old_caller = meta_dispatcher.execute_meta_tx_v0(checker_address, signature.span());
            
                let cheated_address: ContractAddress = 123.try_into().unwrap();
                start_cheat_caller_address(checker_address, cheated_address);

                let meta_result = meta_dispatcher.execute_meta_tx_v0(checker_address, signature.span());

                assert(meta_result == cheated_address.into(), 'Should see cheated addr');

                stop_cheat_caller_address(checker_address);

                let meta_result = meta_dispatcher.execute_meta_tx_v0(checker_address, signature.span());

                assert(meta_result == old_caller, 'Caller should revert back');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatCallerAddressCheckerMetaTxV0".to_string(),
            Path::new("tests/data/contracts/meta_tx_v0_checkers.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "MetaTxV0Test".to_string(),
            Path::new("tests/data/contracts/meta_tx_v0_test.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);
    assert_passed(&result);
}

#[test]
fn meta_tx_v0_with_cheat_block_hash() {
    // TODO(#3704) Remove scarb version check
    if skip_scarb_lt_2_11_0() {
        return;
    }

    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, start_cheat_block_hash,
                stop_cheat_block_hash
            };

            #[starknet::interface]
            trait IMetaTxV0Test<TContractState> {
                fn execute_meta_tx_v0_get_block_hash(
                    ref self: TContractState,
                    target: starknet::ContractAddress,
                    block_number: u64,
                    signature: Span<felt252>,
                ) -> felt252;
            }

            #[test]
            fn test_meta_tx_v0_with_cheat_block_hash() {
                let checker_contract = declare("CheatBlockHashCheckerMetaTxV0").unwrap().contract_class();
                let (checker_address, _) = checker_contract.deploy(@ArrayTrait::new()).unwrap();

                let meta_contract = declare("MetaTxV0Test").unwrap().contract_class();
                let (meta_address, _) = meta_contract.deploy(@ArrayTrait::new()).unwrap();
                let meta_dispatcher = IMetaTxV0TestDispatcher { contract_address: meta_address };

                let block_number = 100;
                let signature = ArrayTrait::new();

                let old_block_hash = meta_dispatcher.execute_meta_tx_v0_get_block_hash(checker_address, block_number, signature.span());

                let cheated_hash = 555;
                start_cheat_block_hash(checker_address, block_number, cheated_hash);

                let meta_result = meta_dispatcher.execute_meta_tx_v0_get_block_hash(checker_address, block_number, signature.span());

                assert(meta_result == cheated_hash, 'Should see cheated hash');

                stop_cheat_block_hash(checker_address, block_number);

                let new_block_hash = meta_dispatcher.execute_meta_tx_v0_get_block_hash(checker_address, block_number, signature.span());
                assert(new_block_hash == old_block_hash, 'Block hash should revert back');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatBlockHashCheckerMetaTxV0".to_string(),
            Path::new("tests/data/contracts/meta_tx_v0_checkers.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "MetaTxV0Test".to_string(),
            Path::new("tests/data/contracts/meta_tx_v0_test.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);
    assert_passed(&result);
}

#[test]
fn meta_tx_v0_verify_tx_context_modification() {
    // TODO(#3704) Remove scarb version check
    if skip_scarb_lt_2_11_0() {
        return;
    }

    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait
            };

            #[starknet::interface]
            trait IMetaTxV0Test<TContractState> {
                fn execute_meta_tx_v0(
                    ref self: TContractState,
                    target: starknet::ContractAddress,
                    signature: Span<felt252>,
                ) -> felt252;
            }

            #[starknet::interface]
            trait ITxInfoCheckerMetaTxV0<TContractState> {
                fn __execute__(ref self: TContractState) -> felt252;
            }

            #[test]
            fn test_meta_tx_v0_verify_tx_context_modification() {
                let checker_contract = declare("TxInfoCheckerMetaTxV0").unwrap().contract_class();
                let (checker_address, _) = checker_contract.deploy(@ArrayTrait::new()).unwrap();
                let checker_dispatcher = ITxInfoCheckerMetaTxV0Dispatcher { contract_address: checker_address };

                let meta_contract = declare("MetaTxV0Test").unwrap().contract_class();
                let (meta_address, _) = meta_contract.deploy(@ArrayTrait::new()).unwrap();
                let meta_dispatcher = IMetaTxV0TestDispatcher { contract_address: meta_address };

                let direct_version = checker_dispatcher.__execute__();

                let mut signature = ArrayTrait::new();

                let meta_version = meta_dispatcher.execute_meta_tx_v0(checker_address, signature.span());

                assert(meta_version == 0, 'Meta tx version should be 0');

                assert(direct_version == 3, 'Direct call version should be 3');
            }
        "#
        ),
        Contract::from_code_path(
            "TxInfoCheckerMetaTxV0".to_string(),
            Path::new("tests/data/contracts/meta_tx_v0_checkers.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "MetaTxV0Test".to_string(),
            Path::new("tests/data/contracts/meta_tx_v0_test.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test, ForgeTrackedResource::CairoSteps);
    assert_passed(&result);
}
