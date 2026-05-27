use crate::response::cast_message::SncastCommandMessage;
use conversions::string::IntoDecStr;
use foundry_ui::styling::OutputBuilder;
use serde::{Serialize, Serializer};
use starknet_rust::core::types::{
    DeclareTransactionReceipt, DeployAccountTransactionReceipt, DeployTransactionReceipt,
    ExecutionResources, ExecutionResult, FeePayment, InvokeTransactionReceipt,
    L1HandlerTransactionReceipt, PriceUnit, ReceiptBlock, TransactionFinalityStatus,
    TransactionReceipt, TransactionReceiptWithBlockInfo,
};
use starknet_types_core::felt::Felt;

#[derive(Clone)]
pub struct TxReceiptResponse(pub TransactionReceiptWithBlockInfo);

impl Serialize for TxReceiptResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl SncastCommandMessage for TxReceiptResponse {
    fn text(&self) -> String {
        let builder = OutputBuilder::new()
            .success_message("Transaction receipt retrieved")
            .blank_line();

        let builder = match &self.0.receipt {
            TransactionReceipt::Invoke(r) => append_invoke(builder, r),
            TransactionReceipt::L1Handler(r) => append_l1_handler(builder, r),
            TransactionReceipt::Declare(r) => append_declare(builder, r),
            TransactionReceipt::Deploy(r) => append_deploy(builder, r),
            TransactionReceipt::DeployAccount(r) => append_deploy_account(builder, r),
        };

        append_block(builder, &self.0.block).build()
    }
}

fn append_invoke(builder: OutputBuilder, r: &InvokeTransactionReceipt) -> OutputBuilder {
    append_common(
        builder,
        "INVOKE",
        &r.transaction_hash,
        &r.actual_fee,
        r.finality_status,
        &r.execution_result,
        &r.execution_resources,
        r.messages_sent.len(),
        r.events.len(),
    )
}

fn append_declare(builder: OutputBuilder, r: &DeclareTransactionReceipt) -> OutputBuilder {
    append_common(
        builder,
        "DECLARE",
        &r.transaction_hash,
        &r.actual_fee,
        r.finality_status,
        &r.execution_result,
        &r.execution_resources,
        r.messages_sent.len(),
        r.events.len(),
    )
}

fn append_deploy(builder: OutputBuilder, r: &DeployTransactionReceipt) -> OutputBuilder {
    append_common(
        builder,
        "DEPLOY",
        &r.transaction_hash,
        &r.actual_fee,
        r.finality_status,
        &r.execution_result,
        &r.execution_resources,
        r.messages_sent.len(),
        r.events.len(),
    )
    .padded_felt_field("Contract Address", &r.contract_address)
}

fn append_deploy_account(
    builder: OutputBuilder,
    r: &DeployAccountTransactionReceipt,
) -> OutputBuilder {
    append_common(
        builder,
        "DEPLOY ACCOUNT",
        &r.transaction_hash,
        &r.actual_fee,
        r.finality_status,
        &r.execution_result,
        &r.execution_resources,
        r.messages_sent.len(),
        r.events.len(),
    )
    .padded_felt_field("Contract Address", &r.contract_address)
}

fn append_l1_handler(builder: OutputBuilder, r: &L1HandlerTransactionReceipt) -> OutputBuilder {
    append_common(
        builder,
        "L1 HANDLER",
        &r.transaction_hash,
        &r.actual_fee,
        r.finality_status,
        &r.execution_result,
        &r.execution_resources,
        r.messages_sent.len(),
        r.events.len(),
    )
    .field("Message Hash", &format!("{}", r.message_hash))
}

#[expect(clippy::too_many_arguments)]
fn append_common(
    builder: OutputBuilder,
    tx_type: &str,
    transaction_hash: &Felt,
    actual_fee: &FeePayment,
    finality_status: TransactionFinalityStatus,
    execution_result: &ExecutionResult,
    execution_resources: &ExecutionResources,
    messages_sent: usize,
    events: usize,
) -> OutputBuilder {
    let builder = builder
        .field("Type", tx_type)
        .padded_felt_field("Transaction Hash", transaction_hash)
        .field("Finality Status", fmt_finality(finality_status))
        .field("Execution Status", fmt_execution_status(execution_result));

    let builder = if let ExecutionResult::Reverted { reason } = execution_result {
        builder.field("Revert Reason", reason)
    } else {
        builder
    };

    builder
        .field(
            "Actual Fee",
            &format!(
                "{} {}",
                actual_fee.amount.into_dec_string(),
                fmt_price_unit(actual_fee.unit)
            ),
        )
        .field("L1 Gas Consumed", &execution_resources.l1_gas.to_string())
        .field(
            "L1 Data Gas Consumed",
            &execution_resources.l1_data_gas.to_string(),
        )
        .field("L2 Gas Consumed", &execution_resources.l2_gas.to_string())
        .field("Messages Sent", &messages_sent.to_string())
        .field("Events", &events.to_string())
}

fn append_block(builder: OutputBuilder, block: &ReceiptBlock) -> OutputBuilder {
    match block {
        ReceiptBlock::PreConfirmed { block_number } => {
            builder.field("Block Number", &block_number.to_string())
        }
        ReceiptBlock::Block {
            block_hash,
            block_number,
        } => builder
            .padded_felt_field("Block Hash", block_hash)
            .field("Block Number", &block_number.to_string()),
    }
}

fn fmt_finality(status: TransactionFinalityStatus) -> &'static str {
    match status {
        TransactionFinalityStatus::PreConfirmed => "Pre confirmed",
        TransactionFinalityStatus::AcceptedOnL2 => "Accepted on L2",
        TransactionFinalityStatus::AcceptedOnL1 => "Accepted on L1",
    }
}

fn fmt_execution_status(result: &ExecutionResult) -> &'static str {
    match result {
        ExecutionResult::Succeeded => "Succeeded",
        ExecutionResult::Reverted { .. } => "Reverted",
    }
}

fn fmt_price_unit(unit: PriceUnit) -> &'static str {
    match unit {
        PriceUnit::Wei => "WEI",
        PriceUnit::Fri => "FRI",
    }
}
