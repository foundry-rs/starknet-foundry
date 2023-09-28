use crate::{
    cheatcodes::{CheatcodeError, EnhancedHintError},
    CheatnetState, state::{BlockifierState, ExtendedStateReader},
};
use blockifier::state::{cached_state::CachedState, state_api::State};
use starknet_api::core::{ClassHash, ContractAddress};

impl BlockifierState {
    /// Gets the class hash at the given address.
    pub fn get_class_hash(
        &mut self,
        contract_address: ContractAddress,
    ) -> Result<ClassHash, CheatcodeError> {
        let blockifier_state: &mut dyn State = &mut self.blockifier_state;
        match blockifier_state.get_class_hash_at(contract_address) {
            Ok(class_hash) => Ok(class_hash),
            Err(e) => Err(CheatcodeError::Unrecoverable(EnhancedHintError::State(e))),
        }
    }
}
