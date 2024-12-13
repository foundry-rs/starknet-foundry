use crate::{get_provider, on_empty_message, CastConfig};
use anyhow::Context;
use clap::Args;
use shared::verify_and_warn_if_incompatible_rpc_version;
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient};

#[derive(Args, Clone, Debug, Default)]
pub struct RpcArgs {
    /// RPC provider url address; overrides url from snfoundry.toml
    #[clap(short, long)]
    pub url: Option<String>,
}

impl RpcArgs {
    pub async fn get_provider(
        &self,
        config: &CastConfig,
    ) -> anyhow::Result<JsonRpcClient<HttpTransport>> {
        let url = self
            .url
            .as_ref()
            .or(config.url.as_ref())
            .with_context(|| on_empty_message("url"))?;
        let provider = get_provider(url)?;

        verify_and_warn_if_incompatible_rpc_version(&provider, &url).await?;

        Ok(provider)
    }
}
