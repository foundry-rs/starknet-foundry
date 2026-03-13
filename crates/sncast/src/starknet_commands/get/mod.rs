use clap::{Args, Subcommand};
use sncast::helpers::configuration::CastConfig;
use sncast::response::ui::UI;

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
    TxStatus(tx_status::TxStatus),
}

pub async fn get(get: Get, config: CastConfig, ui: &UI) -> anyhow::Result<()> {
    match get.command {
        GetCommands::TxStatus(status) => tx_status::tx_status(status, config, ui).await?,
    }

    Ok(())
}
