use crate::{from_thru_felt252, FromConv};
use blockifier::execution::execution_utils::felt_to_stark_felt;
use cairo_felt::Felt252;
use starknet::core::types::FieldElement;
use starknet_api::core::{ClassHash, ContractAddress, EntryPointSelector, Nonce};
use starknet_api::hash::StarkFelt;

impl FromConv<Felt252> for StarkFelt {
    fn from_(value: Felt252) -> StarkFelt {
        felt_to_stark_felt(&value)
    }
}

from_thru_felt252!(FieldElement, StarkFelt);
from_thru_felt252!(ClassHash, StarkFelt);
from_thru_felt252!(ContractAddress, StarkFelt);
from_thru_felt252!(Nonce, StarkFelt);
from_thru_felt252!(EntryPointSelector, StarkFelt);
