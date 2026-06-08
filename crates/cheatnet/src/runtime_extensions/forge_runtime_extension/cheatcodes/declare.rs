use crate::constants::get_current_sierra_version;
use crate::runtime_extensions::forge_runtime_extension::{
    cheatcodes::{CheatcodeError, EnhancedHintError},
    contracts_data::{ContractResolutionError, ContractsData},
};
use anyhow::{Result, anyhow};
use blockifier::execution::contract_class::{CompiledClassV1, RunnableCompiledClass};
#[cfg(feature = "cairo-native")]
use blockifier::execution::native::contract_class::NativeCompiledClassV1;
use blockifier::state::{errors::StateError, state_api::State};
use conversions::IntoConv;
use conversions::serde::serialize::CairoSerialize;
use scarb_api::StarknetContractArtifacts;
use starknet_api::core::{ClassHash, CompiledClassHash};
use starknet_rust::core::types::contract::SierraClass;

#[derive(CairoSerialize)]
pub enum DeclareResult {
    Success(ClassHash),
    AlreadyDeclared(ClassHash),
}

pub fn declare(
    state: &mut dyn State,
    contract_name: &str,
    contracts_data: &ContractsData,
) -> Result<DeclareResult, CheatcodeError> {
    let contract = match contracts_data.resolve_by_name(contract_name) {
        Ok(contract) => contract,
        Err(ContractResolutionError::NotFound) => {
            return Err(CheatcodeError::Unrecoverable(EnhancedHintError::from(
                anyhow!("Failed to get contract artifact for name = {contract_name}."),
            )));
        }
        Err(ContractResolutionError::Ambiguous(module_paths)) => {
            let paths = module_paths
                .iter()
                .map(|path| format!("    {path}"))
                .collect::<Vec<_>>()
                .join("\n");
            return Err(CheatcodeError::Unrecoverable(EnhancedHintError::from(
                anyhow!(
                    "Multiple contracts found with name = {contract_name}. \
                    Found contracts at the following paths:\n{paths}\n\
                    Rename one of the contracts so that the name is unique."
                ),
            )));
        }
    };

    let contract_class = get_contract_class(&contract.artifacts);

    let class_hash = contract.class_hash;

    match state.get_compiled_class(class_hash) {
        Err(StateError::UndeclaredClassHash(_)) => {
            // Class is undeclared; declare it.

            state
                .set_contract_class(class_hash, contract_class)
                .map_err(EnhancedHintError::from)?;

            // NOTE: Compiled class hash is being set to 0 here
            // because it is currently only used in verification
            // and we haven't found a way to calculate it easily
            state
                .set_compiled_class_hash(class_hash, CompiledClassHash::default())
                .unwrap_or_else(|err| panic!("Failed to set compiled class hash: {err:?}"));
            Ok(DeclareResult::Success(class_hash))
        }
        Err(error) => Err(CheatcodeError::Unrecoverable(EnhancedHintError::State(
            error,
        ))),
        Ok(_) => {
            // Class is already declared, cannot redeclare
            // (i.e., make sure the leaf is uninitialized).
            Ok(DeclareResult::AlreadyDeclared(class_hash))
        }
    }
}

pub fn get_class_hash(sierra_class: &SierraClass) -> Result<ClassHash> {
    Ok(sierra_class.class_hash()?.into_())
}

fn get_contract_class(contract_artifact: &StarknetContractArtifacts) -> RunnableCompiledClass {
    let contract_class =
        CompiledClassV1::try_from((contract_artifact.casm.clone(), get_current_sierra_version()))
            .expect("Failed to read contract class from json");

    #[cfg(feature = "cairo-native")]
    return match &contract_artifact.executor {
        None => RunnableCompiledClass::V1(contract_class),
        Some(executor) => RunnableCompiledClass::V1Native(NativeCompiledClassV1::new(
            executor.clone(),
            contract_class,
        )),
    };
    #[cfg(not(feature = "cairo-native"))]
    RunnableCompiledClass::V1(contract_class)
}
