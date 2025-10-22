use foundry_ui::Message;
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
        format!(
            "Finished partition run: {}/{}",
            self.partition.index(),
            self.partition.total()
        )
    }

    fn json(&self) -> Value {
        json!({
            "partition": format!("{}/{}", self.partition.index(), self.partition.total())
        })
    }
}
