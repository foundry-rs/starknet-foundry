use super::StarknetConversions;
use blockifier::execution::execution_utils::felt_to_stark_felt;
use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;
use starknet::core::types::FieldElement;
use starknet_api::{
    core::{ClassHash, ContractAddress, PatriciaKey},
    hash::{StarkFelt, StarkHash},
};

impl StarknetConversions for Felt252 {
    fn to_felt252(&self) -> Felt252 {
        self.clone()
    }

    fn to_field_element(&self) -> FieldElement {
        FieldElement::from_bytes_be(&self.to_be_bytes()).unwrap()
    }

    fn to_stark_felt(&self) -> StarkFelt {
        felt_to_stark_felt(self)
    }

    fn to_stark_hash(&self) -> StarkHash {
        StarkHash::new(self.to_be_bytes()).unwrap()
    }

    fn to_class_hash(&self) -> ClassHash {
        ClassHash(self.to_stark_felt())
    }

    fn to_contract_address(&self) -> ContractAddress {
        ContractAddress(PatriciaKey::try_from(self.to_stark_felt()).unwrap())
    }

    fn to_short_string(&self) -> String {
        as_cairo_short_string(self).expect("Conversion to short string failed")
    }
}
