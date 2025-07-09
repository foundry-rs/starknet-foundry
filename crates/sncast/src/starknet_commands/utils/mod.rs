use clap::{Args, Subcommand};
use foundry_ui::UI;
use sncast::{helpers::configuration::CastConfig, response::errors::handle_starknet_command_error};

use crate::{
    process_command_result,
    starknet_commands::{self, utils::serialize::Serialize},
};

pub mod serialize;

#[derive(Args)]
#[command(about = "Utility commands for Starknet")]
pub struct Utils {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Serialize(Serialize),
}

pub async fn utils(utils: Utils, config: CastConfig, ui: &UI) -> anyhow::Result<()> {
    match utils.command {
        Commands::Serialize(serialize) => {
            let result = starknet_commands::utils::serialize::serialize(serialize, config, ui)
                .await
                .map_err(handle_starknet_command_error)?;

            process_command_result("serialize", Ok(result), ui, None);
        }
    }

    Ok(())
}
