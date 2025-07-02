use std::fs;

use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Args;
use data_transformer::transform;
use foundry_ui::UI;
use sncast::{
    get_class_hash_by_address, get_contract_class,
    helpers::{configuration::CastConfig, rpc::RpcArgs},
    response::{errors::StarknetCommandError, serialize::SerializeResponse},
};
use starknet::core::{
    types::{ContractClass, contract::AbiEntry},
    utils::get_selector_from_name,
};
use starknet_types_core::felt::Felt;

#[derive(Args, Clone, Debug)]
#[group(
    required = true,
    multiple = false,
    args = ["class_hash", "contract_address", "abi_file"]
)]
pub struct Location {
    /// Class hash of contract which contains the function
    #[arg(short = 'c', long)]
    pub class_hash: Option<Felt>,

    /// Address of contract which contains the function
    #[arg(short = 'd', long)]
    pub contract_address: Option<Felt>,

    /// Path to the file containing ABI of the contract class
    #[arg(long)]
    pub abi_file: Option<Utf8PathBuf>,
}

#[derive(Args, Clone, Debug)]
#[command(about = "Serialize Cairo expressions into calldata")]
pub struct Serialize {
    /// Comma-separated string of Cairo expressions
    #[arg(long, allow_hyphen_values = true)]
    pub arguments: String,

    /// Name of the function whose calldata should be serialized
    #[arg(short, long)]
    pub function: String,

    #[command(flatten)]
    pub location: Location,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

impl Location {
    pub async fn resolve_class_hash(
        &self,
        rpc_args: RpcArgs,
        config: CastConfig,
        ui: &UI,
    ) -> Result<Felt> {
        match (self.class_hash, self.contract_address) {
            (Some(hash), _) => Ok(hash),
            (None, Some(address)) => {
                let provider = rpc_args.get_provider(&config, ui).await?;
                get_class_hash_by_address(&provider, address).await
            }
            (None, None) => {
                unreachable!("Either `--class-hash` or `--contract-address` must be provided")
            }
        }
    }
}

pub async fn serialize(
    serialize_args: Serialize,
    config: CastConfig,
    ui: &UI,
) -> Result<SerializeResponse, StarknetCommandError> {
    let Serialize {
        function,
        arguments,
        rpc,
        location,
        ..
    } = serialize_args;

    let provider = rpc.get_provider(&config, ui).await?;

    let selector = get_selector_from_name(&function)
        .context("Failed to convert entry point selector to FieldElement")?;

    let calldata = if let Some(abi_file) = location.abi_file {
        let abi_str = fs::read_to_string(abi_file).context("Failed to read ABI file")?;
        let abi: Vec<AbiEntry> =
            serde_json::from_str(&abi_str).context("Failed to deserialize ABI from file")?;
        transform(&arguments, &abi, &selector)?
    } else {
        let class_hash = location.resolve_class_hash(rpc, config, ui).await?;
        let contract_class = get_contract_class(class_hash, &provider).await?;

        let sierra_class = match contract_class {
            ContractClass::Sierra(sierra_class) => sierra_class,
            ContractClass::Legacy(_) => {
                return Err(StarknetCommandError::UnknownError(anyhow::anyhow!(
                    "Transformation of arguments is not available for Cairo Zero contracts"
                )));
            }
        };

        let abi: Vec<AbiEntry> = serde_json::from_str(sierra_class.abi.as_str())
            .context("Couldn't deserialize ABI received from network")?;

        transform(&arguments, &abi, &selector)?
    };
    Ok(SerializeResponse { calldata })
}
