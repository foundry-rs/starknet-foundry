use blockifier::state::state_api::State;
use conversions::{FromConv, IntoConv};
use starknet::core::crypto::pedersen_hash;
use starknet::core::utils::get_selector_from_name;
use starknet_api::core::{ContractAddress, EntryPointSelector, PatriciaKey};
use starknet_api::hash::StarkHash;
use starknet_api::state::StorageKey;
use starknet_types_core::felt::Felt;
use starknet_types_core::felt::NonZeroFelt;

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
    state: &mut dyn State,
    target: ContractAddress,
    storage_address: Felt,
    value: Felt,
) -> Result<(), anyhow::Error> {
    state.set_storage_at(target, storage_key(storage_address)?, value.into_())?;
    Ok(())
}

///
/// # Arguments
///
/// * `blockifier_state`: Blockifier state reader
/// * `target`: The address of the contract we want to target
/// * `storage_address`: Storage address of the felt value we want to load
///
/// returns: Result<Vec<Felt>, Error> - a result containing the read data
///
pub fn load(
    state: &mut dyn State,
    target: ContractAddress,
    storage_address: Felt,
) -> Result<Felt, anyhow::Error> {
    Ok(state
        .get_storage_at(target, storage_key(storage_address)?)?
        .into_())
}

/// The address after hashing with pedersen, needs to be taken with a specific modulo value (2^251 - 256)
/// For details see:
/// <https://docs.starknet.io/architecture-and-concepts/smart-contracts/contract-storage>
#[must_use]
fn normalize_storage_address(address: Felt) -> Felt {
    let modulus = NonZeroFelt::from_felt_unchecked(Felt::from(2).pow(251_u128) - Felt::from(256));
    address.mod_floor(&modulus)
}

#[must_use]
pub fn calculate_variable_address(selector: Felt, key: Option<&[Felt]>) -> Felt {
    let mut address: Felt = selector.into_();
    match key {
        None => address.into_(),
        Some(key) => {
            for key_part in key {
                address = pedersen_hash(&address, &((*key_part).into_()));
            }
            normalize_storage_address(address).into_()
        }
    }
}

#[must_use]
pub fn variable_address(var_name: &str) -> Felt {
    calculate_variable_address(selector_from_name(var_name).into_(), None)
}

#[must_use]
pub fn selector_from_name(name: &str) -> EntryPointSelector {
    let selector = get_selector_from_name(name).unwrap();
    selector.into_()
}

pub fn storage_key(storage_address: Felt) -> Result<StorageKey, anyhow::Error> {
    Ok(StorageKey(PatriciaKey::try_from(StarkHash::from_(
        storage_address,
    ))?))
}

#[must_use]
pub fn map_entry_address(var_name: &str, key: &[Felt]) -> Felt {
    calculate_variable_address(selector_from_name(var_name).into_(), Some(key))
}
