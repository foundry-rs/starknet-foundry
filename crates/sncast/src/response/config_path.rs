use crate::response::cast_message::SncastCommandMessage;
use camino::Utf8PathBuf;
use foundry_ui::styling;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct ConfigPathResponse {
    pub local_config: Option<Utf8PathBuf>,
    pub global_config: Option<Utf8PathBuf>,
}

impl SncastCommandMessage for ConfigPathResponse {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .field(
                "Local Config",
                self.local_config.as_ref().map_or("missing", |p| p.as_str()),
            )
            .field(
                "Global Config",
                self.global_config
                    .as_ref()
                    .map_or("missing", |p| p.as_str()),
            )
            .build()
    }
}
