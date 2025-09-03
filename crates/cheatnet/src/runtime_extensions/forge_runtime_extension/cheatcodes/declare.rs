use crate::constants::get_current_sierra_version;
use crate::runtime_extensions::forge_runtime_extension::{
    cheatcodes::{CheatcodeError, EnhancedHintError},
    contracts_data::ContractsData,
};
use anyhow::{Context, Result};
use blockifier::execution::contract_class::{CompiledClassV1, RunnableCompiledClass};
use blockifier::execution::native::contract_class::NativeCompiledClassV1;
use blockifier::state::{errors::StateError, state_api::State};
use conversions::IntoConv;
use conversions::serde::serialize::CairoSerialize;
use scarb_api::StarknetContractArtifacts;
use starknet::core::types::contract::SierraClass;
use starknet_api::core::{ClassHash, CompiledClassHash};

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
    let contract_artifact = contracts_data
        .get_artifacts(contract_name)
        .with_context(|| format!("Failed to get contract artifact for name = {contract_name}."))
        .map_err(EnhancedHintError::from)?;

    let contract_class = get_contract_class(contract_artifact);

    let class_hash = *contracts_data
        .get_class_hash(contract_name)
        .expect("Failed to get class hash");

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
    let contract_class = CompiledClassV1::try_from_json_string(
        &contract_artifact.casm,
        get_current_sierra_version(),
    )
    .expect("Failed to read contract class from json");

    match &contract_artifact.executor {
        None => RunnableCompiledClass::V1(contract_class),
        Some(executor) => RunnableCompiledClass::V1Native(NativeCompiledClassV1::new(
            executor.clone(),
            contract_class,
        )),
    }
}
