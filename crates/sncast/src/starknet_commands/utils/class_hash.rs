use anyhow::Context;
use clap::Args;
use conversions::{IntoConv, byte_array::ByteArray};
use scarb_api::StarknetContractArtifacts;
use sncast::{
    ErrorData,
    response::{class_hash::ClassHashResponse, errors::StarknetCommandError},
};
use starknet::core::types::contract::SierraClass;
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

pub fn get_class_hash(
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

    Ok(ClassHashResponse {
        class_hash: class_hash.into_(),
    })
}
