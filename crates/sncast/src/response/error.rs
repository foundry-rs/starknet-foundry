use foundry_ui::Message;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ResponseError {
    command: String,
    error: String,
}

impl ResponseError {
    #[must_use]
    pub fn new(command: String, error: String) -> Self {
        Self { command, error }
    }
}

impl Message for ResponseError {
    fn text(&self) -> String
    where
        Self: Sized,
    {
        format!(
            "command: {}
error: {}",
            self.command, self.error
        )
    }
}
