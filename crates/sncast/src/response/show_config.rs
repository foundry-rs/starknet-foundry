use camino::Utf8PathBuf;
use serde::Serialize;

use crate::helpers::block_explorer;

use super::{account::create::Decimal, command::CommandResponse};

#[derive(Serialize, Clone)]
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

// TODO(#3391): Update text output to be more user friendly
// impl Message for SncastMessage<ShowConfigResponse> {}
