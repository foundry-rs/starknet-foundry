use super::command::CommandResponse;
use crate::response::cast_message::SncastMessage;
use conversions::serde::serialize::CairoSerialize;
use foundry_ui::Message;
use foundry_ui::styling;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;

#[derive(Serialize, CairoSerialize, Clone)]
pub enum FinalityStatus {
    Received,
    Rejected,
    AcceptedOnL2,
    AcceptedOnL1,
}

#[derive(Serialize, CairoSerialize, Clone)]
pub enum ExecutionStatus {
    Succeeded,
    Reverted,
}

#[derive(Serialize, CairoSerialize, Clone)]
pub struct TransactionStatusResponse {
    pub finality_status: FinalityStatus,
    pub execution_status: Option<ExecutionStatus>,
}

impl CommandResponse for TransactionStatusResponse {}

impl Message for SncastMessage<TransactionStatusResponse> {
    fn text(&self) -> String {
        let finality_status = match &self.command_response.finality_status {
            FinalityStatus::Received => "Received",
            FinalityStatus::Rejected => "Rejected",
            FinalityStatus::AcceptedOnL2 => "Accepted on L2",
            FinalityStatus::AcceptedOnL1 => "Accepted on L1",
        };

        let mut builder = styling::OutputBuilder::new()
            .success_message("Transaction status retrieved")
            .blank_line()
            .field("Finality Status", finality_status);

        if let Some(execution_status) = &self.command_response.execution_status {
            let execution_str = match execution_status {
                ExecutionStatus::Succeeded => "Succeeded",
                ExecutionStatus::Reverted => "Reverted",
            };
            builder = builder.field("Execution Status", execution_str);
        }

        builder.build()
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.command_response).unwrap_or_else(|err| {
            json!({
                "error": "Failed to serialize response",
                "command": self.command,
                "details": err.to_string()
            })
        })
    }
}
