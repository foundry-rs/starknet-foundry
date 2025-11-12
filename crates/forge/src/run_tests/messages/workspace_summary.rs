use console::style;
use forge_runner::test_target_summary::TestTargetSummary;
use forge_runner::tests_summary::TestsSummary;
use foundry_ui::{Message, components::labeled::LabeledMessage};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Serialize)]
pub struct WorkspaceSummaryMessage {
    summary: TestsSummary,
}

impl WorkspaceSummaryMessage {
    pub const LABEL: &str = "Tests summary";

    #[must_use]
    pub fn new(summaries: &[TestTargetSummary], filtered: Option<usize>) -> Self {
        Self {
            summary: TestsSummary::new(summaries, filtered),
        }
    }
}

impl Message for WorkspaceSummaryMessage {
    fn text(&self) -> String {
        let styled_label = style(&Self::LABEL).bold().to_string();
        LabeledMessage::new(&styled_label, &self.summary.format_summary_message()).text()
    }

    fn json(&self) -> Value {
        json!(self)
    }
}
