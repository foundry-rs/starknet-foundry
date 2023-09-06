use std::path::Path;

use crate::integration::common::runner::Contract;
use crate::integration::common::running_tests::run_test_case;
use crate::{assert_passed, test_case};
use indoc::indoc;

#[test]
fn test_storage_access_from_tests() {
    let test = test_case!(indoc!(
        r#"
        #[starknet::contract]
        mod Contract {
            #[storage]
            struct Storage {
                balance: felt252, 
            }
            
            #[generate_trait]
            impl InternalImpl of InternalTrait {
                fn internal_function(self: @ContractState) -> felt252 {
                    self.balance.read()
                }
            }
        }

        use ___PREFIX_FOR_PACKAGE___test_case::Contract::balance::InternalContractMemberStateTrait;

        #[test]
        fn test_internal() {
            let mut state = Contract::contract_state_for_testing();
            state.balance.write(10);
            
            let value = Contract::InternalImpl::internal_function(@state);
            assert(value == 10, 'Incorrect storage value');
        }
    "#
    ),
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}



#[test]
fn test_simple_syscalls() {
    let test = test_case!(indoc!(
        r#"
        use starknet::info::{get_execution_info};
        use result::ResultTrait;
        use box::BoxTrait;
        use starknet::info::TxInfo;
        use serde::Serde;
        use starknet::ContractAddress;
        use array::SpanTrait;
        use snforge_std::{ declare, ContractClassTrait };

        #[starknet::interface]
        trait ISpoofChecker<TContractState> {
            fn get_tx_hash(ref self: TContractState) -> felt252;
            fn get_nonce(ref self: TContractState) -> felt252;
            fn get_account_contract_address(ref self: TContractState) -> ContractAddress;
            fn get_signature(ref self: TContractState) -> Span<felt252>;
            fn get_version(ref self: TContractState) -> felt252;
            fn get_max_fee(ref self: TContractState) -> u128;
            fn get_chain_id(ref self: TContractState) -> felt252;
        }
        #[starknet::interface]
        trait IRollChecker<TContractState> {
            fn get_block_number(ref self: TContractState) -> u64;
        }

        #[starknet::interface]
        trait IWarpChecker<TContractState> {
            fn get_block_timestamp(ref self: TContractState) -> u64;
        }

        #[test]
        fn test_get_execution_info() {
            let exec_info = get_execution_info().unbox();
            assert(exec_info.caller_address.into() == 0, 'Incorrect caller address');
            assert(exec_info.contract_address.into() == 0, 'Incorrect contract address');
            assert(exec_info.entry_point_selector.into() == 1515033776670125170357353230573218612959527887889480774359330962438924478531, 'Incorrect entry point selector');

            let block_info = exec_info.block_info.unbox();

            let contract_roll = declare('RollChecker');
            let contract_address_roll = contract_roll.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher_roll = IRollCheckerDispatcher { contract_address: contract_address_roll };

            let contract_warp = declare('WarpChecker');
            let contract_address_warp = contract_warp.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher_warp = IWarpCheckerDispatcher { contract_address: contract_address_warp };

            assert(dispatcher_roll.get_block_number() == block_info.block_number, 'Invalid block number');
            assert(dispatcher_warp.get_block_timestamp() == block_info.block_timestamp, 'Invalid block timestamp');

            let contract = declare('SpoofChecker');
            let contract_address = contract.deploy(@ArrayTrait::new()).unwrap();
            let dispatcher = ISpoofCheckerDispatcher { contract_address };

            let tx_info = exec_info.tx_info.unbox();
            assert(tx_info.version == dispatcher.get_version(), 'Incorrect version');
            assert(tx_info.account_contract_address == dispatcher.get_account_contract_address(), 'Incorrect acc_address');
            assert(tx_info.max_fee == dispatcher.get_max_fee(), 'Incorrect max fee');
            assert(tx_info.signature == dispatcher.get_signature(), 'Incorrect signature');
            assert(tx_info.transaction_hash == dispatcher.get_tx_hash(), 'Incorrect transaction_hash');
            assert(tx_info.chain_id == dispatcher.get_chain_id(), 'Incorrect chain_id');
            assert(tx_info.nonce == dispatcher.get_nonce(), 'Incorrect nonce');
        }
    "#
    ),
        Contract::from_code_path(
            "SpoofChecker".to_string(),
            Path::new("tests/data/contracts/spoof_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "RollChecker".to_string(),
            Path::new("tests/data/contracts/roll_checker.cairo"),
        )
        .unwrap(),
        Contract::from_code_path(
            "WarpChecker".to_string(),
            Path::new("tests/data/contracts/warp_checker.cairo"),
        )
        .unwrap()
    );

    let result = run_test_case(&test);

    assert_passed!(result);
}
