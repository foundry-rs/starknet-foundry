use crate::response::cast_message::SncastCommandMessage;
use foundry_ui::styling;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct SpecVersionResponse {
    pub spec_version: String,
}

impl SncastCommandMessage for SpecVersionResponse {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Specification version retrieved")
            .blank_line()
            .field("Specification Version", &self.spec_version)
            .build()
    }
}
