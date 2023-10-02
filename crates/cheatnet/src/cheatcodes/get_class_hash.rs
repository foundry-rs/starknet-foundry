use crate::{
    cheatcodes::{CheatcodeError, EnhancedHintError},
    state::BlockifierState,
};
use blockifier::state::state_api::State;
use starknet_api::core::{ClassHash, ContractAddress};

impl BlockifierState<'_> {
    /// Gets the class hash at the given address.
    pub fn get_class_hash(
        &mut self,
        contract_address: ContractAddress,
    ) -> Result<ClassHash, CheatcodeError> {
        let blockifier_state_raw: &mut dyn State = self.blockifier_state;
        match blockifier_state_raw.get_class_hash_at(contract_address) {
            Ok(class_hash) => Ok(class_hash),
            Err(e) => Err(CheatcodeError::Unrecoverable(EnhancedHintError::State(e))),
        }
    }
}
