use crate::Network;
use crate::helpers::block_explorer;
use crate::response::cast_message::SncastCommandMessage;
use camino::Utf8PathBuf;
use foundry_ui::styling;
use serde::Serialize;
use url::Url;

#[derive(Serialize, Clone)]
pub struct ShowConfigResponse {
    pub profile: Option<String>,
    pub chain_id: Option<String>,
    pub rpc_url: Option<Url>,
    pub network: Option<Network>,
    pub account: Option<String>,
    pub accounts_file_path: Option<Utf8PathBuf>,
    pub keystore: Option<Utf8PathBuf>,
    pub wait_timeout: Option<u64>,
    pub wait_retry_interval: Option<u64>,
    pub show_explorer_links: bool,
    pub block_explorer: Option<block_explorer::Service>,
}

impl SncastCommandMessage for ShowConfigResponse {
    fn text(&self) -> String {
        let builder = styling::OutputBuilder::new()
            .if_some(self.profile.as_ref(), |b, profile| {
                b.field("Profile", profile)
            })
            .if_some(self.chain_id.as_ref(), |b, chain_id| {
                b.field("Chain ID", chain_id)
            })
            .if_some(self.rpc_url.as_ref(), |b, rpc_url| {
                b.field("RPC URL", rpc_url.as_ref())
            })
            .if_some(self.network.as_ref(), |b, network| {
                b.field("Network", network.to_string().as_ref())
            })
            .if_some(self.account.as_ref(), |b, account| {
                b.field("Account", account)
            })
            .if_some(self.accounts_file_path.as_ref(), |b, path| {
                b.field("Accounts File Path", path.as_ref())
            })
            .if_some(self.keystore.as_ref(), |b, keystore| {
                b.field("Keystore", keystore.as_ref())
            })
            .if_some(self.wait_timeout.as_ref(), |b, timeout| {
                b.field("Wait Timeout", format!("{}s", &timeout).as_ref())
            })
            .if_some(self.wait_retry_interval.as_ref(), |b, interval| {
                b.field("Wait Retry Interval", format!("{}s", &interval).as_ref())
            })
            .field("Show Explorer Links", &self.show_explorer_links.to_string())
            .if_some(self.block_explorer.as_ref(), |b, explorer| {
                b.field("Block Explorer", &format!("{explorer:?}",))
            });

        builder.build()
    }
}
