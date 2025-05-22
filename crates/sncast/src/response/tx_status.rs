use conversions::serde::serialize::CairoSerialize;
use serde::Serialize;

use super::command::CommandResponse;

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

// TODO(#3391): Update text output to be more user friendly
// impl Message for SnastMessage<TransactionStatusResponse> {}
