use camino::Utf8PathBuf;
use foundry_ui::{Message, formats::OutputFormat};
use serde::Serialize;

use crate::helpers::block_explorer;

use super::{
    account::create::Decimal,
    cast_message::CastMessage,
    command::CommandResponse,
    print::{Format, OutputData},
};

#[derive(Serialize)]
pub struct ShowConfigResponse {
    pub profile: Option<String>,
    pub chain_id: Option<String>,
    pub rpc_url: Option<String>,
    pub account: Option<String>,
    pub accounts_file_path: Option<Utf8PathBuf>,
    pub keystore: Option<Utf8PathBuf>,
    pub wait_timeout: Option<Decimal>,
    pub wait_retry_interval: Option<Decimal>,
    pub show_explorer_links: bool,
    pub block_explorer: Option<block_explorer::Service>,
}

impl CommandResponse for ShowConfigResponse {}

impl Message for ShowConfigResponse {}

impl CastMessage<ShowConfigResponse> {
    // TODO(#3391): Update text output to be more user friendly
    #[must_use]
    pub fn text(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty("show-config", OutputFormat::Human)
            .expect("Failed to format response")
    }

    #[must_use]
    pub fn json(&self) -> String {
        OutputData::from(&self.message)
            .format_with(self.numbers_format)
            .to_string_pretty("show-config", OutputFormat::Json)
            .expect("Failed to format response")
    }
}
