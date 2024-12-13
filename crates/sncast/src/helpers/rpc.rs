use crate::{get_provider, on_empty_message, CastConfig, Network};
use anyhow::{bail, Context, Result};
use clap::Args;
use shared::verify_and_warn_if_incompatible_rpc_version;
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient};
use std::env::current_exe;
use std::time::UNIX_EPOCH;

#[derive(Args, Clone, Debug, Default)]
#[group(required = true, multiple = false)]
pub struct RpcArgs {
    /// RPC provider url address; overrides url from snfoundry.toml
    #[clap(short, long)]
    pub url: Option<String>,

    #[clap(long)]
    pub network: Option<Network>,
}

impl RpcArgs {
    pub async fn get_provider(&self, config: &CastConfig) -> Result<JsonRpcClient<HttpTransport>> {
        if self.network.is_some() && config.url.is_some() {
            bail!("Argument `--network` cannot be used when `url` is defined for a profile")
        }

        let url = if let Some(network) = self.network {
            let free_provider = FreeProvider::semi_random();
            network.url(&free_provider)
        } else {
            let url = self.url.clone().or_else(|| config.url.clone());

            url.with_context(|| on_empty_message("url"))?
        };

        let provider = get_provider(&url)?;

        verify_and_warn_if_incompatible_rpc_version(&provider, url).await?;

        Ok(provider)
    }
}

fn installation_constant_seed() -> Result<u64> {
    let executable_path = current_exe()?;
    let metadata = executable_path.metadata()?;
    let modified_time = metadata.modified()?;
    let duration = modified_time.duration_since(UNIX_EPOCH)?;

    Ok(duration.as_secs())
}

enum FreeProvider {
    Blast,
    Voyager,
}

impl FreeProvider {
    fn semi_random() -> Self {
        let seed = installation_constant_seed().unwrap_or(2);
        if seed % 2 == 0 {
            return Self::Blast;
        }
        Self::Voyager
    }
}

impl Network {
    fn url(self, provider: &FreeProvider) -> String {
        match self {
            Network::Mainnet => Self::free_mainnet_rpc(provider),
            Network::Sepolia => Self::free_sepolia_rpc(provider),
        }
    }

    fn free_mainnet_rpc(provider: &FreeProvider) -> String {
        match provider {
            FreeProvider::Blast => "https://starknet-mainnet.public.blastapi.io".to_string(),
            FreeProvider::Voyager => "https://free-rpc.nethermind.io/mainnet-juno".to_string(),
        }
    }

    fn free_sepolia_rpc(provider: &FreeProvider) -> String {
        match provider {
            FreeProvider::Blast => "https://starknet-sepolia.public.blastapi.io".to_string(),
            FreeProvider::Voyager => "https://free-rpc.nethermind.io/sepolia-juno".to_string(),
        }
    }
}
