use clap::{Args, Subcommand};
use sncast::helpers::configuration::CastConfig;
use sncast::response::ui::UI;
use std::process::ExitCode;

pub mod balance;
pub mod block;
pub mod class_hash_at;
pub mod nonce;
pub mod spec_version;
pub mod transaction;
pub mod tx_receipt;
pub mod tx_status;

#[derive(Args)]
#[command(about = "Commands for querying Starknet state")]
pub struct Get {
    #[command(subcommand)]
    pub command: GetCommands,
}

#[derive(Debug, Subcommand)]
pub enum GetCommands {
    /// Get the status of a transaction
    #[command(alias = "transaction-status")]
    TxStatus(tx_status::TxStatus),

    /// Get the receipt of a transaction
    #[command(alias = "transaction-receipt")]
    TxReceipt(tx_receipt::TxReceipt),

    /// Get the transaction by hash
    #[command(alias = "transaction")]
    Tx(transaction::Transaction),

    /// Fetch balance of the account for specified token
    Balance(balance::Balance),

    /// Get nonce of a contract
    Nonce(nonce::Nonce),

    /// Get class hash of a contract at a given address
    ClassHashAt(class_hash_at::ClassHashAt),

    /// Get version of Starknet JSON-RPC specification used by the node
    SpecVersion(spec_version::SpecVersion),

    /// Get a block with transaction hashes
    Block(block::Block),
}

pub async fn get(get: Get, config: CastConfig, ui: &UI) -> anyhow::Result<ExitCode> {
    match get.command {
        GetCommands::TxStatus(status) => tx_status::tx_status(status, config, ui).await,

        GetCommands::Tx(tx) => transaction::transaction(tx, config, ui).await,

        GetCommands::TxReceipt(tx) => tx_receipt::tx_receipt(tx, config, ui).await,

        GetCommands::Balance(balance) => balance::balance(balance, config, ui).await,

        GetCommands::Nonce(nonce) => nonce::nonce(nonce, config, ui).await,

        GetCommands::ClassHashAt(args) => class_hash_at::class_hash_at(args, config, ui).await,

        GetCommands::SpecVersion(args) => spec_version::spec_version(args, config, ui).await,

        GetCommands::Block(args) => block::block(args, config, ui).await,
    }
}
