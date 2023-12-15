use crate::state::BlockifierState;
use cairo_felt::Felt252;
use conversions::{FromConv, IntoConv};
use num_traits::Pow;
use starknet::core::types::FieldElement;
use starknet_api::core::{ContractAddress, PatriciaKey};
use starknet_api::hash::StarkHash;
use starknet_api::state::StorageKey;

///
/// # Arguments
///
/// * `blockifier_state`: Blockifier state reader
/// * `target`: The address of the contract we want to target
/// * `storage_address`: Beginning of the storage of the variable
/// * `size`: How many felts we want to read from the calculated offset (`target` + `storage_address`)
///
/// returns: Result<Vec<Felt252>, Error> - a result containing the read data  
///
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

/// The address after hashing with pedersen, needs to be taken with a specific modulo value (2^251 - 256)
/// For details see:
/// <https://docs.starknet.io/documentation/architecture_and_concepts/Smart_Contracts/contract-storage>
#[must_use]
pub fn map_storage_address(address: FieldElement) -> FieldElement {
    let modulus = Felt252::from(2).pow(251) - Felt252::from(256);
    address % modulus.into_()
}
