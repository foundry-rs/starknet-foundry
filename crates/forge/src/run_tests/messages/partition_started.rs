use console::style;
use foundry_ui::{Message, components::labeled::LabeledMessage};
use serde::Serialize;
use serde_json::{Value, json};

use crate::partition::Partition;

#[derive(Serialize)]
pub struct PartitionStartedMessage {
    partition: Partition,
}

impl PartitionStartedMessage {
    #[must_use]
    pub fn new(partition: Partition) -> Self {
        Self { partition }
    }
}

impl Message for PartitionStartedMessage {
    fn text(&self) -> String {
        let styled_label = style("Running partition run").bold().to_string();
        LabeledMessage::new(
            &styled_label,
            &format!("{}/{}", self.partition.index(), self.partition.total()),
        )
        .text()
    }

    fn json(&self) -> Value {
        json!(self)
    }
}
