use clap::Args;
use shared::verify_and_warn_if_incompatible_rpc_version;
use sncast::{get_provider, helpers::configuration::CastConfig};
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
        let url = self.url.as_ref().unwrap_or(&config.url).to_owned();
        let provider = get_provider(&url)?;

        verify_and_warn_if_incompatible_rpc_version(&provider, &url).await?;

        Ok(provider)
    }
}
