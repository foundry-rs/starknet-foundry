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
pub struct LocationArgs {
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

#[derive(Debug)]
pub enum Location {
    AbiFile(Utf8PathBuf),
    ClassHash(Felt),
    ContractAddress(Felt),
}

impl TryFrom<LocationArgs> for Location {
    type Error = anyhow::Error;

    fn try_from(args: LocationArgs) -> Result<Self> {
        match (args.class_hash, args.contract_address, args.abi_file) {
            (Some(class_hash), None, None) => Ok(Location::ClassHash(class_hash)),
            (None, Some(address), None) => Ok(Location::ContractAddress(address)),
            (None, None, Some(path)) => Ok(Location::AbiFile(path)),
            _ => bail!(
                "Exactly one of --class-hash, --contract-address, or --abi-file must be provided"
            ),
        }
    }
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
    pub location_args: LocationArgs,

    #[command(flatten)]
    pub rpc_args: Option<RpcArgs>,
}

pub async fn serialize(
    Serialize {
        function,
        arguments,
        rpc_args,
        location_args,
    }: Serialize,
    config: CastConfig,
    ui: &UI,
) -> Result<SerializeResponse, StarknetCommandError> {
    let selector = get_selector_from_name(&function)
        .context("Failed to convert entry point selector to FieldElement")?;
    let location = Location::try_from(location_args)?;

    let abi = resolve_abi(location, rpc_args, &config, ui).await?;

    let calldata = transform(&arguments, &abi, &selector)?;

    Ok(SerializeResponse { calldata })
}

pub async fn resolve_abi(
    location: Location,
    rpc_args: Option<RpcArgs>,
    config: &CastConfig,
    ui: &UI,
) -> Result<Vec<AbiEntry>> {
    match location {
        Location::AbiFile(path) => {
            let abi_str = tokio::fs::read_to_string(path)
                .await
                .context("Failed to read ABI file")?;
            serde_json::from_str(&abi_str).context("Failed to deserialize ABI from file")
        }
        Location::ClassHash(class_hash) => {
            let provider = rpc_args
                .context(
                    "Either `--network` or `--url` must be provided when using `--class-hash`",
                )?
                .get_provider(config, ui)
                .await?;
            let contract_class = get_contract_class(class_hash, &provider).await?;
            parse_abi_from_contract_class(contract_class)
        }
        Location::ContractAddress(address) => {
            let provider = rpc_args.context("Either `--network` or `--url` must be provided when using `--contract-address`")?.get_provider(config, ui).await?;
            let class_hash = get_class_hash_by_address(&provider, address).await?;
            let contract_class = get_contract_class(class_hash, &provider).await?;
            parse_abi_from_contract_class(contract_class)
        }
    }
}

fn parse_abi_from_contract_class(contract_class: ContractClass) -> Result<Vec<AbiEntry>> {
    let ContractClass::Sierra(sierra) = contract_class else {
        bail!("ABI transformation not supported for Cairo 0 (legacy) contracts");
    };
    serde_json::from_str(&sierra.abi).context("Couldn't deserialize ABI from network")
}
