use super::StarknetConversions;
use blockifier::execution::execution_utils::stark_felt_to_felt;
use cairo_felt::Felt252;
use starknet::core::types::FieldElement;
use starknet_api::{
    core::{ClassHash, ContractAddress},
    hash::{StarkFelt, StarkHash},
};
use starknet_api::core::Nonce;

impl StarknetConversions for ClassHash {
    fn to_felt252(&self) -> Felt252 {
        stark_felt_to_felt(self.0)
    }

    fn to_field_element(&self) -> FieldElement {
        self.to_felt252().to_field_element()
    }

    fn to_stark_felt(&self) -> StarkFelt {
        self.to_felt252().to_stark_felt()
    }

    fn to_stark_hash(&self) -> StarkHash {
        self.to_felt252().to_stark_hash()
    }

    fn to_class_hash(&self) -> ClassHash {
        *self
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
