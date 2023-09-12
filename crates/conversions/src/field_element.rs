use super::StarknetConversions;
use cairo_felt::Felt252;
use starknet::core::types::FieldElement;
use starknet_api::core::Nonce;
use starknet_api::{
    core::{ClassHash, ContractAddress},
    hash::{StarkFelt, StarkHash},
};

impl StarknetConversions for FieldElement {
    fn to_felt252(&self) -> Felt252 {
        Felt252::from_bytes_be(&self.to_bytes_be())
    }

    fn to_field_element(&self) -> FieldElement {
        *self
    }

    fn to_stark_felt(&self) -> StarkFelt {
        self.to_felt252().to_stark_felt()
    }

    fn to_stark_hash(&self) -> StarkHash {
        self.to_felt252().to_stark_hash()
    }

    fn to_class_hash(&self) -> ClassHash {
        self.to_felt252().to_class_hash()
    }

    fn to_contract_address(&self) -> ContractAddress {
        self.to_felt252().to_contract_address()
    }

    fn to_short_string(&self) -> String {
        self.to_felt252().to_short_string()
    }

    fn to_nonce(&self) -> Nonce {
        self.to_felt252().to_nonce()
    }
}
