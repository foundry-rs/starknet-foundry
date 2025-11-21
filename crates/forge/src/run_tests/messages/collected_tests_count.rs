use console::style;
use foundry_ui::Message;
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Serialize)]
pub struct CollectedTestsCountMessage {
    pub tests_num: usize,
    pub package_name: String,
}

impl Message for CollectedTestsCountMessage {
    fn text(&self) -> String {
        let full = format!(
            "\n\nCollected {} test(s) from {} package",
            self.tests_num, self.package_name
        );
        style(full).bold().to_string()
    }

    fn json(&self) -> Value {
        json!(self)
    }
}
