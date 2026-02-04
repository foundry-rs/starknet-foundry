use console::style;
use forge_runner::{test_target_summary::TestTargetSummary, tests_summary::TestsSummary};
use foundry_ui::{Message, components::labeled::LabeledMessage};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Serialize)]
pub struct TestsSummaryMessage {
    summary: TestsSummary,
}

impl TestsSummaryMessage {
    pub const LABEL: &str = "Tests";

    #[must_use]
    pub fn new(summaries: &[TestTargetSummary], filtered: Option<usize>, skipped: Option<usize>) -> Self {
        Self {
            summary: TestsSummary::new(summaries, filtered, skipped),
        }
    }
}

impl Message for TestsSummaryMessage {
    fn text(&self) -> String {
        let styled_label = style(&Self::LABEL).bold().to_string();
        LabeledMessage::new(&styled_label, &self.summary.format_summary_message()).text()
    }

    fn json(&self) -> Value {
        json!(self)
    }
}
