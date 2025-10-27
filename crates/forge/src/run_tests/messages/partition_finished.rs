use console::style;
use foundry_ui::{Message, components::labeled::LabeledMessage};
use serde::Serialize;
use serde_json::{Value, json};

use crate::partition::Partition;

#[derive(Serialize)]
pub struct PartitionFinishedMessage {
    partition: Partition,
}

impl PartitionFinishedMessage {
    #[must_use]
    pub fn new(partition: Partition) -> Self {
        Self { partition }
    }
}

impl Message for PartitionFinishedMessage {
    fn text(&self) -> String {
        let styled_label = style("Finished partition run").bold().to_string();
        LabeledMessage::new(
            &styled_label,
            &format!("{}/{}", self.partition.index, self.partition.total),
        )
        .text()
    }

    fn json(&self) -> Value {
        json!(self)
    }
}
