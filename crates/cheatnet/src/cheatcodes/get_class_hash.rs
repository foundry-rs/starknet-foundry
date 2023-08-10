use crate::{
    cheatcodes::{CheatcodeError, EnhancedHintError},
    CheatnetState,
};
use starknet_api::core::{ContractAddress, ClassHash};
use blockifier::state::{
    state_api::StateReader,
};

impl CheatnetState {

    /// Gets the class hash at the given address.
    pub fn get_class_hash(
        &mut self,
        contract_address: ContractAddress,
    ) -> Result<ClassHash, CheatcodeError> {
        match self.blockifier_state.get_class_hash_at(contract_address) {
            Ok(class_hash) => Ok(class_hash),
            Err(e) => Err(CheatcodeError::Unrecoverable(EnhancedHintError::State(e)))
        }
    }
}
