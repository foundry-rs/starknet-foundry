use crate::response::cast_message::SncastCommandMessage;
use conversions::serde::serialize::CairoSerialize;
use foundry_ui::styling;
use serde::Serialize;

#[derive(Serialize, CairoSerialize, Clone)]
pub enum FinalityStatus {
    Received,
    Candidate,
    PreConfirmed,
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

impl SncastCommandMessage for TransactionStatusResponse {
    fn text(&self) -> String {
        let finality_status = match &self.finality_status {
            FinalityStatus::Received => "Received",
            FinalityStatus::Candidate => "Candidate",
            FinalityStatus::PreConfirmed => "Pre confirmed",
            FinalityStatus::AcceptedOnL2 => "Accepted on L2",
            FinalityStatus::AcceptedOnL1 => "Accepted on L1",
        };

        let mut builder = styling::OutputBuilder::new()
            .success_message("Transaction status retrieved")
            .blank_line()
            .field("Finality Status", finality_status);

        if let Some(execution_status) = &self.execution_status {
            let execution_str = match execution_status {
                ExecutionStatus::Succeeded => "Succeeded",
                ExecutionStatus::Reverted => "Reverted",
            };
            builder = builder.field("Execution Status", execution_str);
        }

        builder.build()
    }
}
