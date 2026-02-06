use console::style;
use forge_runner::partition::Partition;
use foundry_ui::{Message, components::labeled::LabeledMessage};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Serialize)]
pub struct PartitionFinishedMessage {
    partition: Partition,
    included_tests_count: usize,
    excluded_tests_count: usize,
}

impl PartitionFinishedMessage {
    #[must_use]
    pub fn new(
        partition: Partition,
        included_tests_count: usize,
        excluded_tests_count: usize,
    ) -> Self {
        Self {
            partition,
            included_tests_count,
            excluded_tests_count,
        }
    }

    fn summary(&self) -> String {
        format!(
            "{}/{}, included {} out of total {} tests",
            self.partition.index(),
            self.partition.total(),
            self.included_tests_count,
            self.excluded_tests_count + self.included_tests_count
        )
    }
}

impl Message for PartitionFinishedMessage {
    fn text(&self) -> String {
        let label = style("Finished partition run").bold().to_string();
        LabeledMessage::new(&label, &self.summary()).text()
    }

    fn json(&self) -> Value {
        json!(self)
    }
}

#[derive(Serialize)]
pub struct PartitionStartedMessage {
    partition: Partition,
}

impl PartitionStartedMessage {
    #[must_use]
    pub fn new(partition: Partition) -> Self {
        Self { partition }
    }

    fn summary(&self) -> String {
        format!("{}/{}", self.partition.index(), self.partition.total())
    }
}

impl Message for PartitionStartedMessage {
    fn text(&self) -> String {
        let label = style("Running partition run").bold().to_string();
        LabeledMessage::new(&label, &self.summary()).text()
    }

    fn json(&self) -> Value {
        json!(self)
    }
}
