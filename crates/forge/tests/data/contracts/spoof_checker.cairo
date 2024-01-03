use starknet::info::TxInfo;
use serde::Serde;
use option::OptionTrait;
use array::ArrayTrait;
use starknet::ContractAddress;
use starknet::info::v2::ResourceBounds;

#[starknet::interface]
trait ISpoofChecker<TContractState> {
    fn get_tx_hash(self: @TContractState) -> felt252;
    fn get_nonce(self: @TContractState) -> felt252;
    fn get_account_contract_address(self: @TContractState) -> ContractAddress;
    fn get_signature(self: @TContractState) -> Span<felt252>;
    fn get_version(self: @TContractState) -> felt252;
    fn get_max_fee(self: @TContractState) -> u128;
    fn get_chain_id(self: @TContractState) -> felt252;
    fn get_resource_bounds(self: @TContractState) -> Span<ResourceBounds>;
    fn get_tip(self: @TContractState) -> u128;
    fn get_paymaster_data(self: @TContractState) -> Span<felt252>;
    fn get_nonce_data_availability_mode(self: @TContractState) -> u32;
    fn get_fee_data_availability_mode(self: @TContractState) -> u32;
    fn get_account_deployment_data(self: @TContractState) -> Span<felt252>;
    fn get_tx_info(self: @TContractState) -> starknet::info::v2::TxInfo;
}

#[starknet::contract]
mod SpoofChecker {
    use serde::Serde;
    use starknet::info::TxInfo;
    use box::BoxTrait;
    use starknet::ContractAddress;
    use starknet::info::v2::ResourceBounds;
    use starknet::{SyscallResultTrait, SyscallResult, syscalls::get_execution_info_v2_syscall};

    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[abi(embed_v0)]
    impl ISpoofChecker of super::ISpoofChecker<ContractState> {
        fn get_tx_hash(self: @ContractState) -> felt252 {
            starknet::get_tx_info().unbox().transaction_hash
        }

        fn get_nonce(self: @ContractState) -> felt252 {
            starknet::get_tx_info().unbox().nonce
        }

        fn get_account_contract_address(self: @ContractState) -> ContractAddress {
            starknet::get_tx_info().unbox().account_contract_address
        }

        fn get_signature(self: @ContractState) -> Span<felt252> {
            starknet::get_tx_info().unbox().signature
        }

        fn get_version(self: @ContractState) -> felt252 {
            starknet::get_tx_info().unbox().version
        }

        fn get_max_fee(self: @ContractState) -> u128 {
            starknet::get_tx_info().unbox().max_fee
        }

        fn get_chain_id(self: @ContractState) -> felt252 {
            starknet::get_tx_info().unbox().chain_id
        }

        fn get_resource_bounds(self: @ContractState) -> Span<ResourceBounds> {
            get_tx_info_v2().unbox().resource_bounds
        }

        fn get_tip(self: @ContractState) -> u128 {
            get_tx_info_v2().unbox().tip
        }

        fn get_paymaster_data(self: @ContractState) -> Span<felt252> {
            get_tx_info_v2().unbox().paymaster_data
        }

        fn get_nonce_data_availability_mode(self: @ContractState) -> u32 {
            get_tx_info_v2().unbox().nonce_data_availability_mode
        }

        fn get_fee_data_availability_mode(self: @ContractState) -> u32 {
            get_tx_info_v2().unbox().fee_data_availability_mode
        }

        fn get_account_deployment_data(self: @ContractState) -> Span<felt252> {
            get_tx_info_v2().unbox().account_deployment_data
        }

        fn get_tx_info(self: @ContractState) -> starknet::info::v2::TxInfo {
            get_tx_info_v2().unbox()
        }
    }

    fn get_execution_info_v2() -> Box<starknet::info::v2::ExecutionInfo> {
        get_execution_info_v2_syscall().unwrap_syscall()
    }

    fn get_tx_info_v2() -> Box<starknet::info::v2::TxInfo> {
        get_execution_info_v2().unbox().tx_info
    }
}
