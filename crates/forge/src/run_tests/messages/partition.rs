use console::style;
use foundry_ui::{Message, components::labeled::LabeledMessage};
use serde::Serialize;
use serde_json::{Value, json};

use crate::partition::Partition;

#[derive(Serialize)]
pub struct PartitionMessage {
    partition: Partition,
}

impl PartitionMessage {
    #[must_use]
    pub fn new(partition: Partition) -> Self {
        Self { partition }
    }
}

impl Message for PartitionMessage {
    fn text(&self) -> String {
        let styled_label = style("Finished partition run:").bold().to_string();
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
