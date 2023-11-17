use crate::FromConv;
use cairo_felt::Felt252;
use starknet_api::core::{ContractAddress, PatriciaKey};
use starknet_api::hash::StarkHash;

impl FromConv<Felt252> for ContractAddress {
    fn from_(value: Felt252) -> ContractAddress {
        ContractAddress(PatriciaKey::try_from(StarkHash::from_(value)).unwrap())
    }
}
