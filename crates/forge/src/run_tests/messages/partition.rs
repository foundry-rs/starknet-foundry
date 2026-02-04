use crate::partition::Partition;
use console::style;
use foundry_ui::{Message, components::labeled::LabeledMessage};
use serde::Serialize;
use serde_json::{Value, json};

fn partition_summary(partition: &Partition) -> String {
    format!("{}/{}", partition.index(), partition.total())
}

#[derive(Serialize)]
pub struct PartitionFinishedMessage {
    partition: Partition,
}

impl PartitionFinishedMessage {
    #[must_use]
    pub fn new(partition: Partition) -> Self {
        Self { partition }
    }

    fn summary(&self) -> String {
        partition_summary(&self.partition)
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
        partition_summary(&self.partition)
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
