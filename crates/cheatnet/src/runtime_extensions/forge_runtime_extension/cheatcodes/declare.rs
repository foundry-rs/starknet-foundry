use crate::runtime_extensions::forge_runtime_extension::cheatcodes::{
    CheatcodeError, EnhancedHintError,
};
use anyhow::{anyhow, Context, Result};
use bimap::BiMap;
use blockifier::{
    execution::contract_class::{ContractClass as BlockifierContractClass, ContractClassV1},
    state::{errors::StateError, state_api::State},
};
use conversions::IntoConv;
use scarb_api::StarknetContractArtifacts;
use starknet::core::types::contract::SierraClass;
use starknet_api::core::ClassHash;
use std::collections::HashMap;

#[allow(clippy::implicit_hasher)]
pub fn declare(
    state: &mut dyn State,
    contract_name: &str,
    contracts: &HashMap<String, StarknetContractArtifacts>,
    class_hashes: &BiMap<String, ClassHash>,
) -> Result<ClassHash, CheatcodeError> {
    let contract_artifact = contracts.get(contract_name).with_context(|| {
            format!("Failed to get contract artifact for name = {contract_name}. Make sure starknet target is correctly defined in Scarb.toml file.")
        }).map_err::<EnhancedHintError, _>(From::from)?;

    let contract_class = ContractClassV1::try_from_json_string(&contract_artifact.casm)
        .expect("Failed to read contract class from json");
    let contract_class = BlockifierContractClass::V1(contract_class);

    let class_hash = *class_hashes
        .get_by_left(contract_name)
        .expect("Failed to get class hash");

    match state.get_compiled_contract_class(class_hash) {
        Err(StateError::UndeclaredClassHash(_)) => {
            // Class is undeclared; declare it.

            state
                .set_contract_class(class_hash, contract_class)
                .map_err(EnhancedHintError::from)?;

            // NOTE: Compiled class hash is being set to 0 here
            // because it is currently only used in verification
            // and we haven't found a way to calculate it easily
            state
                .set_compiled_class_hash(class_hash, Default::default())
                .unwrap_or_else(|err| panic!("Failed to set compiled class hash: {err:?}"));
            Ok(class_hash)
        }
        Err(error) => Err(CheatcodeError::Unrecoverable(EnhancedHintError::State(
            error,
        ))),
        Ok(_) => {
            // Class is already declared, cannot redeclare
            // (i.e., make sure the leaf is uninitialized).
            Err(CheatcodeError::Unrecoverable(EnhancedHintError::Anyhow(
                anyhow!("Class hash {} is already declared", class_hash),
            )))
        }
    }
}

pub fn get_class_hash(sierra_class: &SierraClass) -> Result<ClassHash> {
    Ok(sierra_class.class_hash()?.into_())
}
