use anyhow::{Context, Result, bail};
use camino::Utf8PathBuf;
use clap::Args;
use foundry_ui::UI;
use sncast::{
    get_class_hash_by_address, get_contract_class,
    helpers::{configuration::CastConfig, rpc::RpcArgs},
    response::{errors::StarknetCommandError, serialize::SerializeResponse},
};
use starknet::core::utils::get_selector_from_name;
use starknet_types_core::felt::Felt;

use crate::Arguments;

#[derive(Args, Clone, Debug)]
#[command(about = "Serialize Cairo expressions into calldata")]
pub struct Serialize {
    /// Comma-separated string of Cairo expressions
    #[arg(long, allow_hyphen_values = true)]
    pub arguments: String,

    /// Class hash of contract which contains the function
    #[arg(short = 'c', long, conflicts_with_all = ["contract_address", "abi_file"])]
    pub class_hash: Option<Felt>,

    /// Address of contract which contains the function
    #[arg(short = 'd', long, conflicts_with_all = ["class_hash", "abi_file"])]
    pub contract_address: Option<Felt>,

    /// Path to the file containing ABI of the contract class
    #[arg(long, conflicts_with_all = ["class_hash", "contract_address"])]
    pub abi_file: Option<Utf8PathBuf>,

    /// Name of the function whose calldata should be serialized
    #[arg(short, long)]
    pub function: String,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

impl Serialize {
    pub async fn class_hash(&self, config: CastConfig, ui: &UI) -> Result<Felt> {
        match (self.class_hash, self.contract_address) {
            (Some(hash), _) => Ok(hash),
            (None, Some(address)) => {
                let provider = self.rpc.get_provider(&config, ui).await?;
                get_class_hash_by_address(&provider, address).await
            }
            (None, None) => bail!("Either `--class-hash` or `--contract-address` must be provided"),
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
        abi_file,
        ..
    } = serialize_args.clone();

    let provider = rpc.get_provider(&config, ui).await?;

    let selector = get_selector_from_name(&function)
        .context("Failed to convert entry point selector to FieldElement")?;

    let arguments = Arguments {
        calldata: None,
        arguments: Some(arguments),
    };
    let calldata = if let Some(abi_file) = abi_file {
        arguments
            .try_into_calldata(None, &selector, Some(abi_file))
            .context("Failed to transform arguments into calldata")?
    } else {
        let class_hash = serialize_args.class_hash(config, ui).await?;
        let contract_class = get_contract_class(class_hash, &provider).await?;
        arguments.try_into_calldata(Some(contract_class), &selector, None)?
    };
    Ok(SerializeResponse { calldata })
}
