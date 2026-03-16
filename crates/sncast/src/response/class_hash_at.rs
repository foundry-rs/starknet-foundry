use crate::helpers::block_explorer::LinkProvider;
use crate::response::cast_message::SncastCommandMessage;
use crate::response::explorer_link::OutputLink;
use conversions::padded_felt::PaddedFelt;
use conversions::string::IntoPaddedHexStr;
use foundry_ui::styling;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct ClassHashAtResponse {
    pub class_hash: PaddedFelt,
}

impl SncastCommandMessage for ClassHashAtResponse {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Class hash retrieved")
            .blank_line()
            .field("Class Hash", &self.class_hash.into_padded_hex_str())
            .build()
    }
}

impl OutputLink for ClassHashAtResponse {
    const TITLE: &'static str = "class";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        format!("class: {}\n", provider.class(self.class_hash))
    }
}
