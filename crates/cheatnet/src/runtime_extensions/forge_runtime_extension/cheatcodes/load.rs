use crate::state::BlockifierState;
use cairo_felt::Felt252;
use conversions::{FromConv, IntoConv};
use num_traits::Pow;
use starknet::core::types::FieldElement;
use starknet_api::core::{ContractAddress, PatriciaKey};
use starknet_api::hash::StarkHash;
use starknet_api::state::StorageKey;

pub fn load(
    blockifier_state: &mut BlockifierState,
    target: ContractAddress,
    storage_address: &Felt252,
    size: &Felt252,
) -> Result<Vec<Felt252>, anyhow::Error> {
    let mut values: Vec<Felt252> = vec![];
    let mut current_slot = storage_address.clone();

    while current_slot < storage_address + size {
        let storage_value = blockifier_state.blockifier_state.get_storage_at(
            target,
            StorageKey(PatriciaKey::try_from(StarkHash::from_(
                current_slot.clone(),
            ))?),
        );
        values.push(Felt252::from_(storage_value?));
        current_slot += Felt252::from(1);
    }
    Ok(values)
}

#[must_use]
pub fn map_storage_address(address: FieldElement) -> FieldElement {
    let modulus = Felt252::from(2).pow(251) - Felt252::from(256);
    address % modulus.into_()
}
