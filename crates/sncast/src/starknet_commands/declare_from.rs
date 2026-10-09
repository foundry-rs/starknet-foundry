use crate::starknet_commands::declare::{
    DeclareCommonArgs, compile_sierra_to_casm, declare_with_artifacts,
};
use crate::starknet_commands::utils::felt_or_id::ClassHash;
use anyhow::{Context, Result};
use clap::{ArgGroup, Args};
use shared::verify_and_warn_if_incompatible_rpc_version;
use sncast::helpers::artifacts::sierra_class_from_file;
use sncast::helpers::command::process_command_result;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::{FreeProvider, RpcArgs};
use sncast::response::declare::DeclareResponse;
use sncast::response::errors::{
    SNCastProviderError, StarknetCommandError, handle_starknet_command_error,
};
use sncast::response::explorer_link::block_explorer_link_if_allowed;
use sncast::response::ui::UI;
use sncast::{Network, WaitForTx, get_account, get_block_id, get_provider, with_account};
use starknet_rust::core::types::contract::{SierraClass, SierraClassDebugInfo};
use starknet_rust::core::types::{BlockId, ContractClass, FlattenedSierraClass};
use starknet_rust::providers::Provider;
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_types_core::felt::Felt;
use std::path::PathBuf;
use std::process::ExitCode;
use url::Url;

#[derive(Args)]
#[command(
    about = "Declare a contract from either: a Sierra file, or by fetching it from a different Starknet instance",
    long_about = None,
    group(
        ArgGroup::new("contract_source")
            .args(["sierra_file", "class_hash"])
            .required(true)
            .multiple(false)
    )
)]
pub struct DeclareFrom {
    /// Path to the compiled Sierra contract class JSON file
    #[arg(long, conflicts_with_all = ["block_id", "source_url", "source_network"])]
    pub sierra_file: Option<PathBuf>,

    /// Class hash of contract declared on a different Starknet instance (hex, decimal, or @alias from snfoundry.toml)
    #[arg(short = 'g', long, conflicts_with = "no_abi")]
    pub class_hash: Option<ClassHash>,

    #[command(flatten)]
    pub source_rpc: SourceRpcArgs,

    /// Block identifier from which the contract will be fetched.
    /// Possible values: `pre_confirmed`, `latest`, block hash (0x prefixed string)
    /// and block number (u64)
    #[arg(short, long, default_value = "latest")]
    pub block_id: String,

    /// If passed, omits ABI from the declared Sierra class. This changes the resulting class hash
    #[arg(long)]
    pub no_abi: bool,

    #[command(flatten)]
    pub common: DeclareCommonArgs,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

#[derive(Args, Clone, Debug, Default)]
#[group(required = false, multiple = false)]
pub struct SourceRpcArgs {
    /// RPC provider url address
    #[arg(short, long)]
    pub source_url: Option<Url>,

    /// Use predefined network with a public provider. Note that this option may result in rate limits or other unexpected behavior
    #[arg(long)]
    pub source_network: Option<Network>,
}

impl SourceRpcArgs {
    pub async fn get_provider(&self, ui: &UI) -> Result<JsonRpcClient<HttpTransport>> {
        let url = self
            .get_url()
            .await
            .context("Either `--source-network` or `--source-url` must be provided")?;

        let provider = get_provider(&url)?;
        // TODO(#3959) Remove `base_ui`
        verify_and_warn_if_incompatible_rpc_version(&provider, url, ui.base_ui()).await?;

        Ok(provider)
    }

    #[must_use]
    async fn get_url(&self) -> Option<Url> {
        if let Some(network) = self.source_network {
            let free_provider = FreeProvider::semi_random();
            Some(network.url(&free_provider).await.ok()?)
        } else {
            self.source_url.clone()
        }
    }
}

pub enum ContractSource {
    LocalFile {
        sierra_path: PathBuf,
    },
    Network {
        source_provider: JsonRpcClient<HttpTransport>,
        class_hash: Felt,
        block_id: BlockId,
    },
}

pub async fn declare_from(
    args: DeclareFrom,
    wait_config: WaitForTx,
    config: CastConfig,
    ui: &UI,
) -> Result<ExitCode> {
    let contract_source = get_contract_source(
        args.sierra_file,
        &args.block_id,
        args.class_hash.as_ref(),
        &args.source_rpc,
        &config,
        ui,
    )
    .await?;

    let provider = args.rpc.get_provider(&config, ui).await?;
    let account = get_account(&config, &provider, &args.rpc, ui).await?;
    let sierra = get_sierra_class(&contract_source)
        .await
        .map_err(handle_starknet_command_error)?;
    let casm = compile_sierra_to_casm(&sierra)?;

    let result = with_account!(&account, |account| declare_with_artifacts(
        sierra,
        casm,
        args.common,
        args.no_abi,
        account,
        wait_config,
        false,
        ui
    )
    .await)
    .map_err(handle_starknet_command_error)?;

    let response = match result {
        DeclareResponse::Success(response) => Ok(response),
        DeclareResponse::DryRun(response) => {
            return Ok(process_command_result(
                "declare-from",
                Ok(response),
                ui,
                None,
            ));
        }
        DeclareResponse::AlreadyDeclared(_) => {
            unreachable!("Argument `skip_on_already_declared` is false")
        }
    };

    let block_explorer_link =
        block_explorer_link_if_allowed(&response, provider.chain_id().await?, &config).await;

    Ok(process_command_result(
        "declare-from",
        response,
        ui,
        block_explorer_link,
    ))
}

fn flattened_sierra_to_sierra(class: FlattenedSierraClass) -> Result<SierraClass> {
    Ok(SierraClass {
        sierra_program: class.sierra_program,
        sierra_program_debug_info: SierraClassDebugInfo {
            type_names: vec![],
            libfunc_names: vec![],
            user_func_names: vec![],
        },
        contract_class_version: class.contract_class_version,
        entry_points_by_type: class.entry_points_by_type,
        abi: serde_json::from_str(&class.abi)?,
    })
}

#[expect(clippy::result_large_err)]
async fn get_sierra_class(
    contract_source: &ContractSource,
) -> Result<SierraClass, StarknetCommandError> {
    match contract_source {
        ContractSource::LocalFile { sierra_path } => sierra_class_from_file(sierra_path),
        ContractSource::Network {
            source_provider,
            class_hash,
            block_id,
        } => {
            let class = source_provider
                .get_class(*block_id, *class_hash)
                .await
                .map_err(SNCastProviderError::from)
                .map_err(StarknetCommandError::from)?;

            let flattened_sierra = match class {
                ContractClass::Sierra(c) => c,
                ContractClass::Legacy(_) => {
                    return Err(StarknetCommandError::UnknownError(anyhow::anyhow!(
                        "Declaring from Cairo 0 (legacy) contracts is not supported"
                    )));
                }
            };
            let sierra: SierraClass = flattened_sierra_to_sierra(flattened_sierra)
                .expect("Failed to parse flattened sierra class");

            let sierra_class_hash = sierra.class_hash().map_err(anyhow::Error::from)?;

            if *class_hash != sierra_class_hash {
                return Err(StarknetCommandError::UnknownError(anyhow::anyhow!(
                    "The provided sierra class hash {class_hash:#x} does not match the computed class hash {sierra_class_hash:#x} from the fetched contract."
                )));
            }
            Ok(sierra)
        }
    }
}

async fn get_contract_source(
    sierra_file: Option<PathBuf>,
    block_id: &str,
    class_hash: Option<&ClassHash>,
    source_rpc: &SourceRpcArgs,
    config: &CastConfig,
    ui: &UI,
) -> Result<ContractSource> {
    if let Some(sierra_file) = sierra_file {
        Ok(ContractSource::LocalFile {
            sierra_path: sierra_file,
        })
    } else {
        let block_id = get_block_id(block_id)?;
        let class_hash = class_hash.expect("missing class_hash").resolve(config)?;
        let source_provider = source_rpc.get_provider(ui).await?;

        Ok(ContractSource::Network {
            source_provider,
            class_hash,
            block_id,
        })
    }
}
