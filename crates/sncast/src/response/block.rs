use crate::response::cast_message::SncastCommandMessage;
use foundry_ui::styling::OutputBuilder;
use serde::{Serialize, Serializer};
use starknet_rust::core::types::{
    BlockStatus, BlockWithTxHashes, L1DataAvailabilityMode, MaybePreConfirmedBlockWithTxHashes,
    PreConfirmedBlockWithTxHashes, ResourcePrice,
};

#[derive(Clone)]
pub struct BlockResponse(pub MaybePreConfirmedBlockWithTxHashes);

impl Serialize for BlockResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl SncastCommandMessage for BlockResponse {
    fn text(&self) -> String {
        let builder = OutputBuilder::new()
            .success_message("Block retrieved")
            .blank_line();

        let builder = match &self.0 {
            MaybePreConfirmedBlockWithTxHashes::Block(block) => append_block(builder, block),
            MaybePreConfirmedBlockWithTxHashes::PreConfirmedBlock(block) => {
                append_pre_confirmed_block(builder, block)
            }
        };

        builder.build()
    }
}

fn append_block(builder: OutputBuilder, block: &BlockWithTxHashes) -> OutputBuilder {
    builder
        .field("Status", fmt_status(block.status))
        .padded_felt_field("Block Hash", &block.block_hash)
        .field("Block Number", &block.block_number.to_string())
        .padded_felt_field("Parent Hash", &block.parent_hash)
        .padded_felt_field("New Root", &block.new_root)
        .field("Timestamp", &block.timestamp.to_string())
        .padded_felt_field("Sequencer Address", &block.sequencer_address)
        .field("L1 Gas Price", &fmt_resource_price(&block.l1_gas_price))
        .field("L2 Gas Price", &fmt_resource_price(&block.l2_gas_price))
        .field(
            "L1 Data Gas Price",
            &fmt_resource_price(&block.l1_data_gas_price),
        )
        .field("L1 DA Mode", fmt_da_mode(block.l1_da_mode))
        .field("Starknet Version", &block.starknet_version)
        .field("Transaction Count", &block.transactions.len().to_string())
        .felt_list_field("Transactions", &block.transactions)
}

fn append_pre_confirmed_block(
    builder: OutputBuilder,
    block: &PreConfirmedBlockWithTxHashes,
) -> OutputBuilder {
    builder
        .field("Status", "Pre confirmed")
        .field("Block Number", &block.block_number.to_string())
        .field("Timestamp", &block.timestamp.to_string())
        .padded_felt_field("Sequencer Address", &block.sequencer_address)
        .field("L1 Gas Price", &fmt_resource_price(&block.l1_gas_price))
        .field("L2 Gas Price", &fmt_resource_price(&block.l2_gas_price))
        .field(
            "L1 Data Gas Price",
            &fmt_resource_price(&block.l1_data_gas_price),
        )
        .field("L1 DA Mode", fmt_da_mode(block.l1_da_mode))
        .field("Starknet Version", &block.starknet_version)
        .field("Transaction Count", &block.transactions.len().to_string())
        .felt_list_field("Transactions", &block.transactions)
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
