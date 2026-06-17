use anyhow::Context;
use clap::Args;
use conversions::IntoConv;
use sncast::helpers::artifacts::{ContractArtifactsMap, resolve_contract_artifacts};
use sncast::response::{errors::StarknetCommandError, utils::class_hash::ClassHashResponse};
use starknet_rust::core::types::contract::SierraClass;

#[derive(Args, Debug)]
#[command(about = "Generate the class hash of a contract", long_about = None)]
pub struct ClassHashArgs {
    /// Contract name or absolute/partial module tree path
    #[arg(short = 'c', long = "contract-name")]
    pub contract: String,

    /// Specifies scarb package to be used
    #[arg(long)]
    pub package: Option<String>,
}

#[expect(clippy::result_large_err)]
pub fn sierra_class_from_artifacts(
    contract_identifier: &str,
    artifacts: &ContractArtifactsMap,
) -> Result<SierraClass, StarknetCommandError> {
    let contract_artifacts = resolve_contract_artifacts(contract_identifier, artifacts)?;

    let sierra: SierraClass = serde_json::from_str(&contract_artifacts.sierra)
        .context("Failed to parse sierra artifact")?;

    Ok(sierra)
}

#[expect(clippy::result_large_err)]
pub fn get_class_hash(
    class_hash: &ClassHashArgs,
    artifacts: &ContractArtifactsMap,
) -> Result<ClassHashResponse, StarknetCommandError> {
    let sierra = sierra_class_from_artifacts(&class_hash.contract, artifacts)?;
    let class_hash = sierra.class_hash().map_err(anyhow::Error::from)?;

    Ok(ClassHashResponse {
        class_hash: class_hash.into_(),
    })
}
