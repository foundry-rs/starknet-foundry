use crate::CheatnetState;
use cairo_felt::Felt252;
use conversions::IntoConv;
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

pub enum ReplaceBytecodeError {
    ContractNotDeployed,
}

impl IntoConv<Felt252> for ReplaceBytecodeError {
    fn into_(self) -> Felt252 {
        match self {
            ReplaceBytecodeError::ContractNotDeployed => Felt252::from(0),
        }
    }
}
