use crate::CheatnetState;
use cairo_felt::Felt252;
use conversions::serde::serialize::{BufferWriter, CairoSerialize};
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

impl CairoSerialize for ReplaceBytecodeError {
    fn serialize(&self, output: &mut BufferWriter) {
        match self {
            ReplaceBytecodeError::ContractNotDeployed => output.write_felt(Felt252::from(0)),
            ReplaceBytecodeError::UndeclaredClassHash => output.write_felt(Felt252::from(1)),
        }
    }
}
