use starknet::info::TxInfo;
use serde::Serde;
use option::OptionTrait;
use array::ArrayTrait;
use starknet::ContractAddress;

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

#[starknet::contract]
mod SpoofChecker {
    use serde::Serde;
    use starknet::info::TxInfo;
    use box::BoxTrait;
    use starknet::ContractAddress;

    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[abi(embed_v0)]
    impl ISpoofChecker of super::ISpoofChecker<ContractState> {
        fn get_tx_hash(ref self: ContractState) -> felt252 {
            starknet::get_tx_info().unbox().transaction_hash
        }

        fn get_nonce(ref self: ContractState) -> felt252 {
            starknet::get_tx_info().unbox().nonce
        }

        fn get_account_contract_address(ref self: ContractState) -> ContractAddress {
            starknet::get_tx_info().unbox().account_contract_address
        }

        fn get_signature(ref self: ContractState) -> Span<felt252> {
            starknet::get_tx_info().unbox().signature
        }

        fn get_version(ref self: ContractState) -> felt252 {
            starknet::get_tx_info().unbox().version
        }

        fn get_max_fee(ref self: ContractState) -> u128 {
            starknet::get_tx_info().unbox().max_fee
        }

        fn get_chain_id(ref self: ContractState) -> felt252 {
            starknet::get_tx_info().unbox().chain_id
        }
    }
}
