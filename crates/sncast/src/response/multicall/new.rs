use camino::Utf8PathBuf;
use serde::Serialize;

use crate::response::command::CommandResponse;

#[derive(Serialize, Clone)]
pub struct MulticallNewResponse {
    pub path: Utf8PathBuf,
    pub content: String,
}

impl CommandResponse for MulticallNewResponse {}

// TODO(#3391): Update text output to be more user friendly
// impl Message for CastMessage<MulticallNewResponse> {}
