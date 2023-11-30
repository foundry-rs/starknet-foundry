use crate::{FromConv, TryFromConv};
use blockifier::execution::execution_utils::stark_felt_to_felt;
use cairo_felt::{Felt252, ParseFeltError};
use num_traits::Num;
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

impl FromConv<Nonce> for Felt252 {
    fn from_(value: Nonce) -> Felt252 {
        stark_felt_to_felt(value.0)
    }
}

impl TryFromConv<String> for Felt252 {
    type Error = ParseFeltError;

    fn try_from_(value: String) -> Result<Felt252, Self::Error> {
        Felt252::from_str_radix(&value, 10)
    }
}
