use clap::Args;
use sncast::{get_provider_and_verify_rpc_version, helpers::configuration::CastConfig};
use starknet::providers::{jsonrpc::HttpTransport, JsonRpcClient};

#[derive(Args, Clone, Debug, Default)]
pub struct RpcArgs {
    /// RPC provider url address; overrides url from snfoundry.toml
    #[clap(long)]
    pub url: Option<String>,
}

pub trait Provider {
    async fn get_provider(
        &self,
        config: &CastConfig,
    ) -> anyhow::Result<JsonRpcClient<HttpTransport>>;
}

impl Provider for RpcArgs {
    async fn get_provider(
        &self,
        config: &CastConfig,
    ) -> anyhow::Result<JsonRpcClient<HttpTransport>> {
        get_provider_and_verify_rpc_version(self.url.as_ref().unwrap_or(&config.url)).await
    }
}
