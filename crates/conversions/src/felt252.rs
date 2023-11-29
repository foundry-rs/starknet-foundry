use crate::FromConv;
use blockifier::execution::execution_utils::stark_felt_to_felt;
use cairo_felt::Felt252;
use starknet::core::types::FieldElement;
use starknet_api::core::Nonce;
use starknet_api::{
    core::{ClassHash, ContractAddress},
    hash::StarkFelt,
};

impl FromConv<FieldElement> for Felt252 {
    fn from_(value: FieldElement) -> Felt252 {
        Felt252::from_bytes_be(&value.to_bytes_be())
    }
}

impl FromConv<StarkFelt> for Felt252 {
    fn from_(value: StarkFelt) -> Felt252 {
        stark_felt_to_felt(value)
    }
}

impl FromConv<ClassHash> for Felt252 {
    fn from_(value: ClassHash) -> Felt252 {
        Felt252::from_bytes_be(value.0.bytes())
    }
}

impl FromConv<ContractAddress> for Felt252 {
    fn from_(value: ContractAddress) -> Felt252 {
        stark_felt_to_felt(*value.0.key())
    }
}

impl FromConv<String> for Felt252 {
    fn from_(value: String) -> Felt252 {
        Felt252::from_bytes_be(value.as_bytes())
    }
}

impl FromConv<Nonce> for Felt252 {
    fn from_(value: Nonce) -> Felt252 {
        stark_felt_to_felt(value.0)
    }
}
