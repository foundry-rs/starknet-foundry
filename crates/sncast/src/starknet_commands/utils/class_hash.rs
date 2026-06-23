use anyhow::Context;
use clap::{ArgGroup, Args};
use conversions::{IntoConv, byte_array::ByteArray};
use sncast::helpers::artifacts::CastStarknetContractArtifacts;
use sncast::{
    ErrorData,
    response::{errors::StarknetCommandError, utils::class_hash::ClassHashResponse},
};
use starknet_rust::core::types::contract::SierraClass;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Args, Debug)]
#[command(
    about = "Generate the class hash of a contract",
    long_about = None,
    group(ArgGroup::new("contract_source").required(true).multiple(false))
)]
pub struct ClassHashArgs {
    /// Contract name
    #[arg(short = 'c', long = "contract-name", group = "contract_source")]
    pub contract: Option<String>,

    /// Path to the compiled Sierra contract class JSON file
    #[arg(long, group = "contract_source", conflicts_with = "package")]
    pub sierra_file: Option<PathBuf>,

    /// Specifies scarb package to be used
    #[arg(long)]
    pub package: Option<String>,
}

#[expect(clippy::result_large_err)]
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
pub fn sierra_class_from_file(sierra_path: &Path) -> Result<SierraClass, StarknetCommandError> {
    let sierra_json = std::fs::read_to_string(sierra_path)
        .with_context(|| format!("Failed to read sierra file at {}", sierra_path.display()))?;

    let sierra: SierraClass =
        serde_json::from_str(&sierra_json).context("Failed to parse sierra file")?;

    Ok(sierra)
}

#[expect(clippy::result_large_err)]
pub fn get_class_hash(
    class_hash: &ClassHashArgs,
    artifacts: Option<&HashMap<String, CastStarknetContractArtifacts>>,
) -> Result<ClassHashResponse, StarknetCommandError> {
    let sierra = if let Some(sierra_file) = &class_hash.sierra_file {
        sierra_class_from_file(sierra_file)?
    } else {
        let contract_name = class_hash
            .contract
            .as_ref()
            .context("`--contract-name` must be provided when `--sierra-file` is not used")?;
        sierra_class_from_artifacts(
            contract_name,
            artifacts.context("artifacts must be provided when `--contract-name` is used")?,
        )?
    };

    let class_hash = sierra.class_hash().map_err(anyhow::Error::from)?;

    Ok(ClassHashResponse {
        class_hash: class_hash.into_(),
    })
}
