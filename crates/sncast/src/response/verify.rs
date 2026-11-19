use crate::response::cast_message::SncastCommandMessage;
use foundry_ui::{Message, styling::OutputBuilder};
use serde::{Serialize, ser::SerializeStruct};

#[derive(Serialize, Clone)]
pub struct VerifyResponse {
    pub message: String,
}

impl SncastCommandMessage for VerifyResponse {
    fn text(&self) -> String {
        OutputBuilder::new()
            .success_message("Verification completed")
            .blank_line()
            .text_field(&self.message)
            .build()
    }
}

pub struct UploadedFiles(pub Vec<String>);

impl Message for UploadedFiles {
    fn text(&self) -> String {
        let builder =
            OutputBuilder::new().heading("The following files will be uploaded to verifier:");

        let builder = self.0.iter().fold(builder, |b, file| {
            b.indented(2, |b| b.text_field(file.as_str()))
        });

        builder.build()
    }

    fn json(&self) -> serde_json::Value {
        serde_json::to_value(self).expect("Should be serializable to JSON")
    }
}

impl Serialize for UploadedFiles {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("UploadedFiles", 2)?;
        s.serialize_field("message", "following files will be uploaded to verifier")?;
        s.serialize_field("files", &self.0)?;
        s.end()
    }
}
