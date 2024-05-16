use crate::CheatnetState;
use cairo_felt::Felt252;
use conversions::felt252::SerializeAsFelt252Vec;
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
    UndeclaredClassHash,
}

impl SerializeAsFelt252Vec for ReplaceBytecodeError {
    fn serialize_into_felt252_vec(self, output: &mut Vec<Felt252>) {
        match self {
            ReplaceBytecodeError::ContractNotDeployed => output.push(Felt252::from(0)),
            ReplaceBytecodeError::UndeclaredClassHash => output.push(Felt252::from(1)),
        }
    }
}
