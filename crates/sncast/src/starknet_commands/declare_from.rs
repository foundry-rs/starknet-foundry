use crate::starknet_commands::declare::{
    declare_with_artifacts, DeclareCommonArgs,
};
use anyhow::{Context, Result};
use clap::Args;
use shared::verify_and_warn_if_incompatible_rpc_version;
use sncast::helpers::rpc::FreeProvider;
use sncast::response::declare::DeclareResponse;
use sncast::response::errors::{SNCastProviderError, StarknetCommandError};
use sncast::response::ui::UI;
use sncast::{Network, WaitForTx, get_block_id, get_provider};
use starknet_rust::accounts::SingleOwnerAccount;
use starknet_rust::core::types::contract::{
    AbiEntry, CompiledClass, SierraClass, SierraClassDebugInfo,
};
use starknet_rust::core::types::{ContractClass, FlattenedSierraClass};
use starknet_rust::providers::Provider;
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_rust::signers::LocalWallet;
use starknet_types_core::felt::Felt;
use universal_sierra_compiler_api::compile_contract_sierra;
use url::Url;

#[derive(Args)]
#[command(about = "Declare a contract by fetching it from a different Starknet instance", long_about = None)]
pub struct DeclareFrom {
    /// Class hash of contract declared on a different Starknet instance
    #[arg(short = 'g', long)]
    pub class_hash: Felt,

    #[command(flatten)]
    pub source_rpc: SourceRpcArgs,

    /// Block identifier from which the contract will be fetched.
    /// Possible values: `pre_confirmed`, `latest`, block hash (0x prefixed string)
    /// and block number (u64)
    #[arg(short, long, default_value = "latest")]
    pub block_id: String,

    #[command(flatten)]
    pub common: DeclareCommonArgs,

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

pub async fn declare_from(
    declare_from: DeclareFrom,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    wait_config: WaitForTx,
    skip_on_already_declared: bool,
    source_provider: &JsonRpcClient<HttpTransport>,
    ui: &UI,
) -> Result<DeclareResponse, StarknetCommandError> {
    let block_id = get_block_id(&declare_from.block_id)?;
    let class = source_provider
        .get_class(block_id, declare_from.class_hash)
        .await
        .map_err(SNCastProviderError::from)
        .map_err(StarknetCommandError::from)?;

    let flattened_sierra = match class {
        ContractClass::Sierra(class) => class,
        ContractClass::Legacy(_) => {
            return Err(StarknetCommandError::UnknownError(anyhow::anyhow!(
                "Declaring from Cairo 0 (legacy) contracts is not supported"
            )));
        }
    };
    let sierra: SierraClass = flattened_sierra_to_sierra(flattened_sierra)
        .expect("Failed to parse flattened sierra class");

    let casm_json: String = serde_json::to_string(
        &compile_contract_sierra(
            &serde_json::to_value(&sierra).expect("Failed to convert sierra to json value"),
        )
        .expect("Failed to compile sierra to casm"),
    )
    .expect("serialization should succeed");
    let casm: CompiledClass = serde_json::from_str(&casm_json)
        .expect("Failed to deserialize casm JSON into CompiledClass");
    let sierra_class_hash = sierra.class_hash().map_err(anyhow::Error::from)?;

    if declare_from.class_hash != sierra_class_hash {
        return Err(StarknetCommandError::UnknownError(anyhow::anyhow!(
            "The provided sierra class hash {:#x} does not match the computed class hash {:#x} from the fetched contract.",
            declare_from.class_hash,
            sierra_class_hash
        )));
    }

    declare_with_artifacts(
        sierra,
        casm,
        declare_from.common.fee_args,
        declare_from.common.nonce,
        account,
        wait_config,
        skip_on_already_declared,
        ui,
    )
    .await
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
        abi: serde_json::from_str::<Vec<AbiEntry>>(&class.abi)?,
    })
}
