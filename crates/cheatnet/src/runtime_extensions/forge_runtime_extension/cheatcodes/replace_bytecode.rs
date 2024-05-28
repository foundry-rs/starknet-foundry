use crate::CheatnetState;
use conversions::serde::serialize::CairoSerialize;
use starknet_api::core::{ClassHash, ContractAddress};

impl CheatnetState {
    pub fn replace_class_for_contract(
        &mut self,
        contract_address: ContractAddress,
        class_hash: ClassHash,
    ) {
        self.replaced_bytecode_contracts
            .insert(contract_address, class_hash);
    }
}

#[derive(CairoSerialize)]
pub enum ReplaceBytecodeError {
    ContractNotDeployed,
    UndeclaredClassHash,
}
