use crate::state::BlockifierState;
use cairo_felt::Felt252;
use conversions::{FromConv, IntoConv};
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
