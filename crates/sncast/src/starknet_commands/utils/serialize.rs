use anyhow::{Context, Result, bail};
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
#[group(required = true, multiple = false)]
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
    pub rpc: Option<RpcArgs>,
}

impl Location {
    async fn resolve_class_hash(
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

    pub async fn resolve_abi(
        &self,
        rpc: Option<RpcArgs>,
        config: CastConfig,
        ui: &UI,
    ) -> Result<Vec<AbiEntry>> {
        if let Some(ref path) = self.abi_file {
            let abi_str = std::fs::read_to_string(path).context("Failed to read ABI file")?;
            serde_json::from_str(&abi_str).context("Failed to deserialize ABI from file")
        } else {
            let rpc = rpc
                .ok_or_else(|| anyhow::anyhow!("Either `--network` or `--url` must be provided"))?;
            let class_hash = self
                .resolve_class_hash(rpc.clone(), config.clone(), ui)
                .await?;
            let contract_class =
                get_contract_class(class_hash, &rpc.get_provider(&config, ui).await?).await?;

            match contract_class {
                ContractClass::Sierra(sierra) => serde_json::from_str(&sierra.abi)
                    .context("Couldn't deserialize ABI received from network"),
                ContractClass::Legacy(_) => {
                    bail!("Transformation of arguments is not available for Cairo Zero contracts")
                }
            }
        }
    }
}

pub async fn serialize(
    function: String,
    arguments: String,
    rpc: Option<RpcArgs>,
    location: Location,
    config: CastConfig,
    ui: &UI,
) -> Result<SerializeResponse, StarknetCommandError> {
    let selector = get_selector_from_name(&function)
        .context("Failed to convert entry point selector to FieldElement")?;

    let abi = location
        .resolve_abi(rpc, config, ui)
        .await
        .map_err(StarknetCommandError::from)?;

    let calldata = transform(&arguments, &abi, &selector)?;

    Ok(SerializeResponse { calldata })
}
