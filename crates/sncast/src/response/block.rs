use crate::response::cast_message::SncastCommandMessage;
use crate::response::transaction::append_transaction;
use crate::response::tx_receipt::append_receipt;
use foundry_ui::styling::OutputBuilder;
use serde::{Serialize, Serializer};
use starknet_rust::core::types::{
    BlockStatus, L1DataAvailabilityMode, MaybePreConfirmedBlockWithReceipts,
    MaybePreConfirmedBlockWithTxHashes, MaybePreConfirmedBlockWithTxs, ResourcePrice, Transaction,
    TransactionWithReceipt,
};

#[derive(Clone)]
pub enum BlockResponse {
    WithTxHashes(MaybePreConfirmedBlockWithTxHashes),
    WithTxs(MaybePreConfirmedBlockWithTxs),
    WithReceipts(MaybePreConfirmedBlockWithReceipts),
}

impl Serialize for BlockResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            BlockResponse::WithTxHashes(block) => block.serialize(serializer),
            BlockResponse::WithTxs(block) => block.serialize(serializer),
            BlockResponse::WithReceipts(block) => block.serialize(serializer),
        }
    }
}

/// Appends the header fields shared by every confirmed block variant.
macro_rules! append_confirmed_header {
    ($builder:expr, $block:expr) => {
        $builder
            .field("Status", fmt_status($block.status))
            .padded_felt_field("Block Hash", &$block.block_hash)
            .field("Block Number", &$block.block_number.to_string())
            .padded_felt_field("Parent Hash", &$block.parent_hash)
            .padded_felt_field("New Root", &$block.new_root)
            .field("Timestamp", &$block.timestamp.to_string())
            .padded_felt_field("Sequencer Address", &$block.sequencer_address)
            .field("L1 Gas Price", &fmt_resource_price(&$block.l1_gas_price))
            .field("L2 Gas Price", &fmt_resource_price(&$block.l2_gas_price))
            .field(
                "L1 Data Gas Price",
                &fmt_resource_price(&$block.l1_data_gas_price),
            )
            .field("L1 DA Mode", fmt_da_mode($block.l1_da_mode))
            .field("Starknet Version", &$block.starknet_version)
            .field("Transaction Count", &$block.transactions.len().to_string())
    };
}

/// Appends the header fields shared by every pre-confirmed block variant.
macro_rules! append_pre_confirmed_header {
    ($builder:expr, $block:expr) => {
        $builder
            .field("Status", "Pre confirmed")
            .field("Block Number", &$block.block_number.to_string())
            .field("Timestamp", &$block.timestamp.to_string())
            .padded_felt_field("Sequencer Address", &$block.sequencer_address)
            .field("L1 Gas Price", &fmt_resource_price(&$block.l1_gas_price))
            .field("L2 Gas Price", &fmt_resource_price(&$block.l2_gas_price))
            .field(
                "L1 Data Gas Price",
                &fmt_resource_price(&$block.l1_data_gas_price),
            )
            .field("L1 DA Mode", fmt_da_mode($block.l1_da_mode))
            .field("Starknet Version", &$block.starknet_version)
            .field("Transaction Count", &$block.transactions.len().to_string())
    };
}

impl SncastCommandMessage for BlockResponse {
    fn text(&self) -> String {
        let builder = OutputBuilder::new()
            .success_message("Block retrieved")
            .blank_line();

        let builder = match self {
            BlockResponse::WithTxHashes(MaybePreConfirmedBlockWithTxHashes::Block(block)) => {
                append_confirmed_header!(builder, block)
                    .felt_list_field("Transactions", &block.transactions)
            }
            BlockResponse::WithTxHashes(MaybePreConfirmedBlockWithTxHashes::PreConfirmedBlock(
                block,
            )) => append_pre_confirmed_header!(builder, block)
                .felt_list_field("Transactions", &block.transactions),
            BlockResponse::WithTxs(MaybePreConfirmedBlockWithTxs::Block(block)) => {
                append_full_transactions(
                    append_confirmed_header!(builder, block),
                    &block.transactions,
                )
            }
            BlockResponse::WithTxs(MaybePreConfirmedBlockWithTxs::PreConfirmedBlock(block)) => {
                append_full_transactions(
                    append_pre_confirmed_header!(builder, block),
                    &block.transactions,
                )
            }
            BlockResponse::WithReceipts(MaybePreConfirmedBlockWithReceipts::Block(block)) => {
                append_receipts(
                    append_confirmed_header!(builder, block),
                    &block.transactions,
                )
            }
            BlockResponse::WithReceipts(MaybePreConfirmedBlockWithReceipts::PreConfirmedBlock(
                block,
            )) => append_receipts(
                append_pre_confirmed_header!(builder, block),
                &block.transactions,
            ),
        };

        builder.build()
    }
}

fn append_full_transactions(
    mut builder: OutputBuilder,
    transactions: &[Transaction],
) -> OutputBuilder {
    for (index, transaction) in transactions.iter().enumerate() {
        builder = append_transaction(
            builder
                .blank_line()
                .text_field(&format!("Transaction #{}", index + 1)),
            transaction,
        );
    }
    builder
}

fn append_receipts(
    mut builder: OutputBuilder,
    transactions: &[TransactionWithReceipt],
) -> OutputBuilder {
    for (index, transaction) in transactions.iter().enumerate() {
        builder = append_receipt(
            builder
                .blank_line()
                .text_field(&format!("Transaction #{}", index + 1)),
            &transaction.receipt,
        );
    }
    builder
}

fn fmt_status(status: BlockStatus) -> &'static str {
    match status {
        BlockStatus::PreConfirmed => "Pre confirmed",
        BlockStatus::AcceptedOnL2 => "Accepted on L2",
        BlockStatus::AcceptedOnL1 => "Accepted on L1",
    }
}

fn fmt_da_mode(mode: L1DataAvailabilityMode) -> &'static str {
    match mode {
        L1DataAvailabilityMode::Blob => "BLOB",
        L1DataAvailabilityMode::Calldata => "CALLDATA",
    }
}

fn fmt_resource_price(price: &ResourcePrice) -> String {
    format!(
        "price_in_fri={}, price_in_wei={}",
        price.price_in_fri, price.price_in_wei
    )
}
