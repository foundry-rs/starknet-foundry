use camino::Utf8PathBuf;
use foundry_ui::Message;
use foundry_ui::styling;
use serde::Serialize;
use serde_json::Value;

use crate::helpers::block_explorer;

use super::command::CommandResponse;
use crate::response::cast_message::SncastMessage;

#[derive(Serialize, Clone)]
pub struct ShowConfigResponse {
    pub profile: Option<String>,
    pub chain_id: Option<String>,
    pub rpc_url: Option<String>,
    pub account: Option<String>,
    pub accounts_file_path: Option<Utf8PathBuf>,
    pub keystore: Option<Utf8PathBuf>,
    pub wait_timeout: Option<u64>,
    pub wait_retry_interval: Option<u64>,
    pub show_explorer_links: bool,
    pub block_explorer: Option<block_explorer::Service>,
}

impl CommandResponse for ShowConfigResponse {}

impl Message for SncastMessage<ShowConfigResponse> {
    fn text(&self) -> String {
        let mut builder = styling::OutputBuilder::new()
            .success_message("Configuration details")
            .blank_line();

        // Add optional fields conditionally
        builder = builder
            .if_some(self.command_response.profile.as_ref(), |b, profile| {
                b.field("Profile", profile)
            })
            .if_some(self.command_response.chain_id.as_ref(), |b, chain_id| {
                b.field("Chain ID", chain_id)
            })
            .if_some(self.command_response.rpc_url.as_ref(), |b, rpc_url| {
                b.field("RPC URL", rpc_url)
            })
            .if_some(self.command_response.account.as_ref(), |b, account| {
                b.field("Account", account)
            })
            .if_some(
                self.command_response.accounts_file_path.as_ref(),
                |b, path| b.field("Accounts File Path", path.as_ref()),
            )
            .if_some(self.command_response.keystore.as_ref(), |b, keystore| {
                b.field("Keystore", keystore.as_ref())
            })
            .if_some(self.command_response.wait_timeout.as_ref(), |b, timeout| {
                b.field("Wait Timeout", &timeout.to_string())
            })
            .if_some(
                self.command_response.wait_retry_interval.as_ref(),
                |b, interval| b.field("Wait Retry Interval", &interval.to_string()),
            )
            .field(
                "Show Explorer Links",
                &self.command_response.show_explorer_links.to_string(),
            )
            .if_some(
                self.command_response.block_explorer.as_ref(),
                |b, explorer| b.field("Block Explorer", &format!("{explorer:?}",)),
            );

        builder.build()
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.command_response).unwrap()
    }
}
