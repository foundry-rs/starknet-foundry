use crate::response::cast_message::SncastCommandMessage;
use crate::response::transaction::append_transaction;
use crate::response::tx_receipt::append_receipt;
use foundry_ui::styling::OutputBuilder;
use serde::{Serialize, Serializer};
use starknet_rust::core::types::{
    BlockStatus, BlockWithReceipts, BlockWithTxHashes, BlockWithTxs, L1DataAvailabilityMode,
    MaybePreConfirmedBlockWithReceipts, MaybePreConfirmedBlockWithTxHashes,
    MaybePreConfirmedBlockWithTxs, PreConfirmedBlockWithReceipts, PreConfirmedBlockWithTxHashes,
    PreConfirmedBlockWithTxs, ResourcePrice, Transaction, TransactionWithReceipt,
};
use starknet_types_core::felt::Felt;

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

trait BlockOutputBuilder {
    #[must_use]
    fn status(self, status: BlockStatus) -> Self;
    #[must_use]
    fn pre_confirmed_status(self) -> Self;
    #[must_use]
    fn block_hash(self, hash: &Felt) -> Self;
    #[must_use]
    fn block_number(self, number: u64) -> Self;
    #[must_use]
    fn parent_hash(self, hash: &Felt) -> Self;
    #[must_use]
    fn new_root(self, root: &Felt) -> Self;
    #[must_use]
    fn timestamp(self, timestamp: u64) -> Self;
    #[must_use]
    fn sequencer_address(self, addr: &Felt) -> Self;
    #[must_use]
    fn l1_gas_price(self, price: &ResourcePrice) -> Self;
    #[must_use]
    fn l2_gas_price(self, price: &ResourcePrice) -> Self;
    #[must_use]
    fn l1_data_gas_price(self, price: &ResourcePrice) -> Self;
    #[must_use]
    fn l1_da_mode(self, mode: L1DataAvailabilityMode) -> Self;
    #[must_use]
    fn starknet_version(self, version: &str) -> Self;
    #[must_use]
    fn transaction_count(self, count: u64) -> Self;
    #[must_use]
    fn event_count(self, count: u64) -> Self;
    #[must_use]
    fn state_diff_length(self, length: u64) -> Self;
}

impl BlockOutputBuilder for OutputBuilder {
    fn status(self, status: BlockStatus) -> Self {
        self.field("Status", fmt_status(status))
    }
    fn pre_confirmed_status(self) -> Self {
        self.field("Status", "Pre confirmed")
    }
    fn block_hash(self, hash: &Felt) -> Self {
        self.padded_felt_field("Block Hash", hash)
    }
    fn block_number(self, number: u64) -> Self {
        self.field("Block Number", &number.to_string())
    }
    fn parent_hash(self, hash: &Felt) -> Self {
        self.padded_felt_field("Parent Hash", hash)
    }
    fn new_root(self, root: &Felt) -> Self {
        self.padded_felt_field("New Root", root)
    }
    fn timestamp(self, timestamp: u64) -> Self {
        self.field("Timestamp", &timestamp.to_string())
    }
    fn sequencer_address(self, addr: &Felt) -> Self {
        self.padded_felt_field("Sequencer Address", addr)
    }
    fn l1_gas_price(self, price: &ResourcePrice) -> Self {
        self.field("L1 Gas Price", &fmt_resource_price(price))
    }
    fn l2_gas_price(self, price: &ResourcePrice) -> Self {
        self.field("L2 Gas Price", &fmt_resource_price(price))
    }
    fn l1_data_gas_price(self, price: &ResourcePrice) -> Self {
        self.field("L1 Data Gas Price", &fmt_resource_price(price))
    }
    fn l1_da_mode(self, mode: L1DataAvailabilityMode) -> Self {
        self.field("L1 DA Mode", fmt_da_mode(mode))
    }
    fn starknet_version(self, version: &str) -> Self {
        self.field("Starknet Version", version)
    }
    fn transaction_count(self, count: u64) -> Self {
        self.field("Transaction Count", &count.to_string())
    }
    fn event_count(self, count: u64) -> Self {
        self.field("Event Count", &count.to_string())
    }
    fn state_diff_length(self, length: u64) -> Self {
        self.field("State Diff Length", &length.to_string())
    }
}

/// Appends the header fields shared by every confirmed block variant.
///
/// Takes the concrete block type so the exhaustive destructure turns any
/// field added upstream into a compile error instead of silently dropping it.
macro_rules! append_confirmed_header {
    ($builder:expr, $block:expr, $ty:path) => {{
        let $ty {
            status,
            block_hash,
            parent_hash,
            block_number,
            new_root,
            timestamp,
            sequencer_address,
            l1_gas_price,
            l2_gas_price,
            l1_data_gas_price,
            l1_da_mode,
            starknet_version,
            event_count,
            transaction_count,
            state_diff_length,
            // Omit Merkle-Patricia tries in the formatted output.
            // They are still returned when using the `--json` flag.
            // `transactions` is handled separately by the caller.
            transactions: _,
            event_commitment: _,
            transaction_commitment: _,
            receipt_commitment: _,
            state_diff_commitment: _,
        } = $block;
        $builder
            .status(*status)
            .block_hash(block_hash)
            .block_number(*block_number)
            .parent_hash(parent_hash)
            .new_root(new_root)
            .timestamp(*timestamp)
            .sequencer_address(sequencer_address)
            .l1_gas_price(l1_gas_price)
            .l2_gas_price(l2_gas_price)
            .l1_data_gas_price(l1_data_gas_price)
            .l1_da_mode(*l1_da_mode)
            .starknet_version(starknet_version)
            .transaction_count(*transaction_count)
            .event_count(*event_count)
            .state_diff_length(*state_diff_length)
    }};
}

/// Appends the header fields shared by every pre-confirmed block variant.
///
/// Takes the concrete block type so the exhaustive destructure turns any
/// field added upstream into a compile error instead of silently dropping it.
macro_rules! append_pre_confirmed_header {
    ($builder:expr, $block:expr, $ty:path) => {{
        let $ty {
            block_number,
            timestamp,
            sequencer_address,
            l1_gas_price,
            l2_gas_price,
            l1_data_gas_price,
            l1_da_mode,
            starknet_version,
            transactions,
        } = $block;
        $builder
            .pre_confirmed_status()
            .block_number(*block_number)
            .timestamp(*timestamp)
            .sequencer_address(sequencer_address)
            .l1_gas_price(l1_gas_price)
            .l2_gas_price(l2_gas_price)
            .l1_data_gas_price(l1_data_gas_price)
            .l1_da_mode(*l1_da_mode)
            .starknet_version(starknet_version)
            .transaction_count(
                u64::try_from(transactions.len()).expect("transaction count fits in u64"),
            )
    }};
}

impl SncastCommandMessage for BlockResponse {
    fn text(&self) -> String {
        let builder = OutputBuilder::new()
            .success_message("Block retrieved")
            .blank_line();

        let builder = match self {
            BlockResponse::WithTxHashes(MaybePreConfirmedBlockWithTxHashes::Block(block)) => {
                append_confirmed_header!(builder, block, BlockWithTxHashes)
                    .felt_list_field("Transactions", &block.transactions)
            }
            BlockResponse::WithTxHashes(MaybePreConfirmedBlockWithTxHashes::PreConfirmedBlock(
                block,
            )) => append_pre_confirmed_header!(builder, block, PreConfirmedBlockWithTxHashes)
                .felt_list_field("Transactions", &block.transactions),
            BlockResponse::WithTxs(MaybePreConfirmedBlockWithTxs::Block(block)) => {
                append_full_transactions(
                    append_confirmed_header!(builder, block, BlockWithTxs),
                    &block.transactions,
                )
            }
            BlockResponse::WithTxs(MaybePreConfirmedBlockWithTxs::PreConfirmedBlock(block)) => {
                append_full_transactions(
                    append_pre_confirmed_header!(builder, block, PreConfirmedBlockWithTxs),
                    &block.transactions,
                )
            }
            BlockResponse::WithReceipts(MaybePreConfirmedBlockWithReceipts::Block(block)) => {
                append_receipts(
                    append_confirmed_header!(builder, block, BlockWithReceipts),
                    &block.transactions,
                )
            }
            BlockResponse::WithReceipts(MaybePreConfirmedBlockWithReceipts::PreConfirmedBlock(
                block,
            )) => append_receipts(
                append_pre_confirmed_header!(builder, block, PreConfirmedBlockWithReceipts),
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
                .text_field(&format!("Transaction #{}", index + 1))
                .with_indent(2),
            transaction,
        )
        .with_indent(0);
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
                .text_field(&format!("Transaction #{}", index + 1))
                .with_indent(2),
            &transaction.receipt,
        )
        .with_indent(0);
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
