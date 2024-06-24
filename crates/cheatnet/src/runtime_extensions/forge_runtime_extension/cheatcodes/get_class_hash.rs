use crate::runtime_extensions::forge_runtime_extension::cheatcodes::{
    CheatcodeError, EnhancedHintError,
};
use blockifier::state::state_api::State;
use starknet_api::core::{ClassHash, ContractAddress};

/// Gets the class hash at the given address.
pub fn get_class_hash(
    state: &mut dyn State,
    contract_address: ContractAddress,
) -> Result<ClassHash, CheatcodeError> {
    match state.get_class_hash_at(contract_address) {
        Ok(class_hash) => Ok(class_hash),
        Err(e) => Err(CheatcodeError::Unrecoverable(EnhancedHintError::State(e))),
    }
}
