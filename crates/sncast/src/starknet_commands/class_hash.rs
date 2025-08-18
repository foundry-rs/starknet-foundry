use anyhow::Context;
use clap::Args;
use conversions::{IntoConv, byte_array::ByteArray};
use scarb_api::StarknetContractArtifacts;
use sncast::{
    ErrorData,
    response::{
        class_hash::{ClassHashGeneratedResponse, ClassHashResponse},
        errors::StarknetCommandError,
    },
};
use starknet::core::types::contract::{CompiledClass, SierraClass};
use std::collections::HashMap;

#[derive(Args)]
#[command(about = "Generate the class hash of a contract", long_about = None)]
pub struct ClassHash {
    /// Contract name
    #[arg(short = 'c', long = "contract-name")]
    pub contract: String,
}

pub async fn get_class_hash(
    class_hash: ClassHash,
    artifacts: &HashMap<String, StarknetContractArtifacts>,
) -> Result<ClassHashResponse, StarknetCommandError> {
    let contract_artifacts = artifacts.get(&class_hash.contract).ok_or(
        StarknetCommandError::ContractArtifactsNotFound(ErrorData {
            data: ByteArray::from(class_hash.contract.as_str()),
        }),
    )?;

    let contract_definition: SierraClass = serde_json::from_str(&contract_artifacts.sierra)
        .context("Failed to parse sierra artifact")?;

    let class_hash = contract_definition
        .class_hash()
        .map_err(anyhow::Error::from)?;

    Ok(ClassHashResponse::Success(ClassHashGeneratedResponse {
        class_hash: class_hash.into_(),
    }))
}
