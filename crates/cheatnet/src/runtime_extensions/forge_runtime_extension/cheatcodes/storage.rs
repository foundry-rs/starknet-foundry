use crate::state::BlockifierState;
use cairo_felt::Felt252;
use conversions::{FromConv, IntoConv};
use num_traits::Pow;
use starknet::core::crypto::pedersen_hash;
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
/// * `values`: A vector of values to write starting at `storage_address`
///
/// returns: Result<(), Error> - a result containing the error if `store` failed  
///
pub fn store(
    blockifier_state: &mut BlockifierState,
    target: ContractAddress,
    storage_address: &Felt252,
    values: &[Felt252],
) -> Result<(), anyhow::Error> {
    for (i, value) in values.iter().enumerate() {
        blockifier_state.blockifier_state.set_storage_at(
            target,
            StorageKey(PatriciaKey::try_from(StarkHash::from_(
                storage_address.clone() + Felt252::from(i),
            ))?),
            value.clone().into_(),
        );
    }
    Ok(())
}

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
fn normalize_storage_address(address: FieldElement) -> FieldElement {
    let modulus = Felt252::from(2).pow(251) - Felt252::from(256);
    address % modulus.into_()
}

#[must_use]
pub fn calculate_variable_address(selector: Felt252, key: Option<&[Felt252]>) -> Felt252 {
    let mut address: FieldElement = selector.into_();
    match key {
        None => address.into_(),
        Some(key) => {
            for key_part in key {
                address = pedersen_hash(&address, &(key_part.clone().into_()));
            }
            normalize_storage_address(address).into_()
        }
    }
}
