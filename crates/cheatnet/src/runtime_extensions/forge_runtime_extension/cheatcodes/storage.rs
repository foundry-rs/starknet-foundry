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
/// * `storage_address`: Storage address of the felt value we want to store
/// * `value`: A felt value to write at `storage_address`
///
/// returns: Result<(), Error> - a result containing the error if `store` failed  
///
pub fn store(
    blockifier_state: &mut BlockifierState,
    target: ContractAddress,
    storage_address: &Felt252,
    value: Felt252,
) -> Result<(), anyhow::Error> {
    blockifier_state.blockifier_state.set_storage_at(
        target,
        storage_key(storage_address)?,
        value.into_(),
    );
    Ok(())
}

///
/// # Arguments
///
/// * `blockifier_state`: Blockifier state reader
/// * `target`: The address of the contract we want to target
/// * `storage_address`: Storage address of the felt value we want to load
///
/// returns: Result<Vec<Felt252>, Error> - a result containing the read data  
///
pub fn load(
    blockifier_state: &mut BlockifierState,
    target: ContractAddress,
    storage_address: &Felt252,
) -> Result<Felt252, anyhow::Error> {
    Ok(blockifier_state
        .blockifier_state
        .get_storage_at(target, storage_key(storage_address)?)?
        .into_())
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
pub fn calculate_variable_address(selector: &Felt252, key: Option<&[Felt252]>) -> Felt252 {
    let mut address: FieldElement = selector.clone().into_();
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

fn storage_key(storage_address: &Felt252) -> Result<StorageKey, anyhow::Error> {
    Ok(StorageKey(PatriciaKey::try_from(StarkHash::from_(
        storage_address.clone(),
    ))?))
}
