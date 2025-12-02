use crate::response::cast_message::SncastCommandMessage;
use camino::Utf8PathBuf;
use foundry_ui::styling;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct MulticallNewResponse {
    pub path: Utf8PathBuf,
    pub content: String,
}

impl SncastCommandMessage for MulticallNewResponse {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Multicall template created successfully")
            .blank_line()
            .field("Path", self.path.as_ref())
            .field("Content", self.content.as_ref())
            .build()
    }
}
