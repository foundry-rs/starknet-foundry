use anyhow::Context;
use clap::Args;
use conversions::{IntoConv, byte_array::ByteArray};
use sncast::helpers::artifacts::CastStarknetContractArtifacts;
use sncast::{
    ErrorData,
    response::{errors::StarknetCommandError, utils::class_hash::ClassHashResponse},
};
use starknet_rust::core::types::contract::SierraClass;
use std::collections::HashMap;

#[derive(Args, Debug)]
#[command(about = "Generate the class hash of a contract", long_about = None)]
pub struct ClassHash {
    /// Contract name
    #[arg(short = 'c', long = "contract-name")]
    pub contract: String,

    /// Specifies scarb package to be used
    #[arg(long)]
    pub package: Option<String>,
}

#[allow(clippy::result_large_err)]
pub fn sierra_class_from_artifacts(
    contract_name: &str,
    artifacts: &HashMap<String, CastStarknetContractArtifacts>,
) -> Result<SierraClass, StarknetCommandError> {
    let contract_artifacts =
        artifacts
            .get(contract_name)
            .ok_or(StarknetCommandError::ContractArtifactsNotFound(ErrorData {
                data: ByteArray::from(contract_name),
            }))?;

    let sierra: SierraClass = serde_json::from_str(&contract_artifacts.sierra)
        .context("Failed to parse sierra artifact")?;

    Ok(sierra)
}

#[expect(clippy::result_large_err)]
pub fn get_class_hash(
    class_hash: &ClassHash,
    artifacts: &HashMap<String, CastStarknetContractArtifacts>,
) -> Result<ClassHashResponse, StarknetCommandError> {
    let sierra = sierra_class_from_artifacts(&class_hash.contract, artifacts)?;
    let class_hash = sierra.class_hash().map_err(anyhow::Error::from)?;

    Ok(ClassHashResponse {
        class_hash: class_hash.into_(),
    })
}
