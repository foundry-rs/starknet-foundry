use clap::{Args, Subcommand};
use sncast::response::ui::UI;
use sncast::{
    helpers::{
        configuration::CastConfig,
        scarb_utils::{
            BuildConfig, assert_manifest_path_exists, build_and_load_artifacts,
            get_package_metadata,
        },
    },
    response::errors::handle_starknet_command_error,
};

use crate::{
    process_command_result,
    starknet_commands::{
        self,
        utils::{
            class_hash::ClassHash,
            contract_address::ContractAddress,
            selector::Selector,
            serialize::Serialize,
        },
    },
};

pub mod class_hash;
pub mod contract_address;
pub mod felt_or_id;
pub mod selector;
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

    /// Get contract class hash
    ClassHash(ClassHash),

    /// Calculate the address of a not yet deployed contract
    ContractAddress(ContractAddress),

    /// Calculate selector from name
    Selector(Selector),
}

pub async fn utils(
    utils: Utils,
    config: CastConfig,
    ui: &UI,
    json: bool,
    profile: String,
) -> anyhow::Result<()> {
    match utils.command {
        Commands::Serialize(serialize) => {
            let result = starknet_commands::utils::serialize::serialize(serialize, config, ui)
                .await
                .map_err(handle_starknet_command_error)?;

            process_command_result("serialize", Ok(result), ui, None);
        }

        Commands::ClassHash(class_hash) => {
            let manifest_path = assert_manifest_path_exists()?;
            let package_metadata = get_package_metadata(&manifest_path, &class_hash.package)?;

            let artifacts = build_and_load_artifacts(
                &package_metadata,
                &BuildConfig {
                    scarb_toml_path: manifest_path,
                    json,
                    profile,
                },
                false,
                // TODO(#3959) Remove `base_ui`
                ui.base_ui(),
            )
            .expect("Failed to build contract");

            let result = class_hash::get_class_hash(&class_hash, &artifacts)
                .map_err(handle_starknet_command_error)?;

            process_command_result("class-hash", Ok(result), ui, None);
        }

        Commands::ContractAddress(contract_address) => {
            let artifacts = if contract_address
                .common
                .contract_identifier
                .contract_name
                .is_some()
            {
                let manifest_path = assert_manifest_path_exists()?;
                let package_metadata =
                    get_package_metadata(&manifest_path, &contract_address.common.package)?;

                Some(
                    build_and_load_artifacts(
                        &package_metadata,
                        &BuildConfig {
                            scarb_toml_path: manifest_path,
                            json,
                            profile,
                        },
                        false,
                        // TODO(#3959) Remove `base_ui`
                        ui.base_ui(),
                    )
                    .expect("Failed to build contract"),
                )
            } else {
                None
            };

            let result =
                contract_address::get_contract_address(contract_address, artifacts, config, ui)
                    .await
                    .map_err(handle_starknet_command_error)?;

            process_command_result("contract-address", Ok(result), ui, None);
        }

        Commands::Selector(sel) => {
            let result = selector::get_selector(&sel).map_err(handle_starknet_command_error)?;
            process_command_result("selector", Ok(result), ui, None);
        }
    }

    Ok(())
}
