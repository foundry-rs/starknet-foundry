use forge_runner::forge_config::ForgeTrackedResource;
use indoc::indoc;
use std::path::Path;
use test_utils::runner::{Contract, assert_passed};
use test_utils::running_tests::run_test_case;
use test_utils::test_case;

#[test]
fn meta_tx_v0_with_cheat_caller_address() {
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
                fn execute_meta_tx_get_caller(
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

                let old_caller = meta_dispatcher.execute_meta_tx_get_caller(checker_address, signature.span());
            
                let cheated_address: ContractAddress = 123.try_into().unwrap();
                start_cheat_caller_address(checker_address, cheated_address);

                let meta_result = meta_dispatcher.execute_meta_tx_get_caller(checker_address, signature.span());

                assert(meta_result == 123, 'Should see cheated addr');

                stop_cheat_caller_address(checker_address);

                let meta_result = meta_dispatcher.execute_meta_tx_get_caller(checker_address, signature.span());

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
fn meta_tx_v0_with_cheat_block_number() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, start_cheat_block_number,
                stop_cheat_block_number
            };

            #[starknet::interface]
            trait IMetaTxV0Test<TContractState> {
                fn execute_meta_tx_get_block_number(
                    ref self: TContractState,
                    target: starknet::ContractAddress,
                    signature: Span<felt252>,
                ) -> felt252;
            }

            #[test]
            fn test_meta_tx_v0_with_cheat_block_number() {
                let checker_contract = declare("CheatBlockNumberCheckerMetaTxV0").unwrap().contract_class();
                let (checker_address, _) = checker_contract.deploy(@ArrayTrait::new()).unwrap();

                let meta_contract = declare("MetaTxV0Test").unwrap().contract_class();
                let (meta_address, _) = meta_contract.deploy(@ArrayTrait::new()).unwrap();
                let meta_dispatcher = IMetaTxV0TestDispatcher { contract_address: meta_address };

                let signature = ArrayTrait::new();

                let old_block_number = meta_dispatcher.execute_meta_tx_get_block_number(checker_address, signature.span());

                let cheated_block_number = 999;
                start_cheat_block_number(checker_address, cheated_block_number);

                let meta_result = meta_dispatcher.execute_meta_tx_get_block_number(checker_address, signature.span());

                assert(meta_result == 999, 'Should see cheated block');

                stop_cheat_block_number(checker_address);

                let new_block_number = meta_dispatcher.execute_meta_tx_get_block_number(checker_address, signature.span());
                assert(new_block_number == old_block_number, 'Block num should revert back');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatBlockNumberCheckerMetaTxV0".to_string(),
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
                fn execute_meta_tx_get_block_hash(
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

                let old_block_hash = meta_dispatcher.execute_meta_tx_get_block_hash(checker_address, block_number, signature.span());

                let cheated_hash = 555;
                start_cheat_block_hash(checker_address, block_number, cheated_hash);

                let meta_result = meta_dispatcher.execute_meta_tx_get_block_hash(checker_address, block_number, signature.span());

                assert(meta_result == 555, 'Should see cheated hash');

                stop_cheat_block_hash(checker_address, block_number);

                let new_block_hash = meta_dispatcher.execute_meta_tx_get_block_hash(checker_address, block_number, signature.span());
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
fn meta_tx_v0_with_cheat_sequencer_address() {
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
                declare, ContractClassTrait, DeclareResultTrait, start_cheat_sequencer_address,
                stop_cheat_sequencer_address
            };

            #[starknet::interface]
            trait IMetaTxV0Test<TContractState> {
                fn execute_meta_tx_get_sequencer_address(
                    ref self: TContractState,
                    target: starknet::ContractAddress,
                    signature: Span<felt252>,
                ) -> felt252;
            }

            #[test]
            fn test_meta_tx_v0_with_cheat_sequencer_address() {
                let checker_contract = declare("CheatSequencerAddressCheckerMetaTxV0").unwrap().contract_class();
                let (checker_address, _) = checker_contract.deploy(@ArrayTrait::new()).unwrap();

                let meta_contract = declare("MetaTxV0Test").unwrap().contract_class();
                let (meta_address, _) = meta_contract.deploy(@ArrayTrait::new()).unwrap();
                let meta_dispatcher = IMetaTxV0TestDispatcher { contract_address: meta_address };

                let signature = ArrayTrait::new();

                let old_sequencer = meta_dispatcher.execute_meta_tx_get_sequencer_address(checker_address, signature.span());

                let cheated_address: ContractAddress = 777.try_into().unwrap();
                start_cheat_sequencer_address(checker_address, cheated_address);

                let meta_result = meta_dispatcher.execute_meta_tx_get_sequencer_address(checker_address, signature.span());

                assert(meta_result == 777, 'Should see cheated seq');

                stop_cheat_sequencer_address(checker_address);

                let new_sequencer = meta_dispatcher.execute_meta_tx_get_sequencer_address(checker_address, signature.span());
                assert(new_sequencer == old_sequencer, 'Seq addr should revert back');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatSequencerAddressCheckerMetaTxV0".to_string(),
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
fn meta_tx_v0_with_cheat_block_timestamp() {
    let test = test_case!(
        indoc!(
            r#"
            use result::ResultTrait;
            use array::ArrayTrait;
            use option::OptionTrait;
            use traits::TryInto;
            use starknet::ContractAddress;
            use snforge_std::{
                declare, ContractClassTrait, DeclareResultTrait, start_cheat_block_timestamp,
                stop_cheat_block_timestamp
            };

            #[starknet::interface]
            trait IMetaTxV0Test<TContractState> {
                fn execute_meta_tx_get_block_timestamp(
                    ref self: TContractState,
                    target: starknet::ContractAddress,
                    signature: Span<felt252>,
                ) -> felt252;
            }

            #[test]
            fn test_meta_tx_v0_with_cheat_block_timestamp() {
                let checker_contract = declare("CheatBlockTimestampCheckerMetaTxV0").unwrap().contract_class();
                let (checker_address, _) = checker_contract.deploy(@ArrayTrait::new()).unwrap();

                let meta_contract = declare("MetaTxV0Test").unwrap().contract_class();
                let (meta_address, _) = meta_contract.deploy(@ArrayTrait::new()).unwrap();
                let meta_dispatcher = IMetaTxV0TestDispatcher { contract_address: meta_address };

                let signature = ArrayTrait::new();

                let old_timestamp = meta_dispatcher.execute_meta_tx_get_block_timestamp(checker_address, signature.span());

                let cheated_timestamp = 1234567890;
                start_cheat_block_timestamp(checker_address, cheated_timestamp);

                let meta_result = meta_dispatcher.execute_meta_tx_get_block_timestamp(checker_address, signature.span());

                assert(meta_result == 1234567890, 'Should see cheated time');

                stop_cheat_block_timestamp(checker_address);

                let new_timestamp = meta_dispatcher.execute_meta_tx_get_block_timestamp(checker_address, signature.span());
                assert(new_timestamp == old_timestamp, 'Timestamp should revert back');
            }
        "#
        ),
        Contract::from_code_path(
            "CheatBlockTimestampCheckerMetaTxV0".to_string(),
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
