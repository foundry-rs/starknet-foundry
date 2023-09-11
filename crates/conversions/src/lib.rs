use cairo_felt::Felt252;
use starknet::core::types::FieldElement;
use starknet_api::core::Nonce;
use starknet_api::{
    core::{ClassHash, ContractAddress},
    hash::{StarkFelt, StarkHash},
};

pub mod class_hash;
pub mod contract_address;
pub mod felt252;
pub mod field_element;
pub mod nonce;
pub mod short_string;
pub mod stark_felt;

pub trait StarknetConversions {
    fn to_felt252(&self) -> Felt252;
    fn to_field_element(&self) -> FieldElement;
    fn to_stark_felt(&self) -> StarkFelt;
    fn to_stark_hash(&self) -> StarkHash; // Alias to StarkFelt
    fn to_class_hash(&self) -> ClassHash;
    fn to_contract_address(&self) -> ContractAddress;
    fn to_short_string(&self) -> String;
    fn to_nonce(&self) -> Nonce;
}
