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
    let BlockWithTxHashes {
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
        transactions,
        transaction_count,
        // Omit Merkle-Patricia tries and redundant information from the formatted output.
        // They are still returned when using the `--json` flag.
        event_commitment: _,
        transaction_commitment: _,
        receipt_commitment: _,
        state_diff_commitment: _,
        event_count: _,
        state_diff_length: _,
    } = block;
    builder
        .field("Status", fmt_status(*status))
        .padded_felt_field("Block Hash", block_hash)
        .field("Block Number", &block_number.to_string())
        .padded_felt_field("Parent Hash", parent_hash)
        .padded_felt_field("New Root", new_root)
        .field("Timestamp", &timestamp.to_string())
        .padded_felt_field("Sequencer Address", sequencer_address)
        .field("L1 Gas Price", &fmt_resource_price(l1_gas_price))
        .field("L2 Gas Price", &fmt_resource_price(l2_gas_price))
        .field("L1 Data Gas Price", &fmt_resource_price(l1_data_gas_price))
        .field("L1 DA Mode", fmt_da_mode(*l1_da_mode))
        .field("Starknet Version", starknet_version)
        .field("Transaction Count", &transaction_count.to_string())
        .felt_list_field("Transactions", transactions)
}

fn append_pre_confirmed_block(
    builder: OutputBuilder,
    block: &PreConfirmedBlockWithTxHashes,
) -> OutputBuilder {
    let PreConfirmedBlockWithTxHashes {
        transactions,
        block_number,
        timestamp,
        sequencer_address,
        l1_gas_price,
        l2_gas_price,
        l1_data_gas_price,
        l1_da_mode,
        starknet_version,
    } = block;
    builder
        .field("Status", "Pre confirmed")
        .field("Block Number", &block_number.to_string())
        .field("Timestamp", &timestamp.to_string())
        .padded_felt_field("Sequencer Address", sequencer_address)
        .field("L1 Gas Price", &fmt_resource_price(l1_gas_price))
        .field("L2 Gas Price", &fmt_resource_price(l2_gas_price))
        .field("L1 Data Gas Price", &fmt_resource_price(l1_data_gas_price))
        .field("L1 DA Mode", fmt_da_mode(*l1_da_mode))
        .field("Starknet Version", starknet_version)
        .field("Transaction Count", &transactions.len().to_string())
        .felt_list_field("Transactions", transactions)
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
