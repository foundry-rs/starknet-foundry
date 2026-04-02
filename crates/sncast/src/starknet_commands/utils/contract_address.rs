use anyhow::Context;
use clap::Args;
use conversions::IntoConv;
use data_transformer::transform;
use sncast::helpers::artifacts::CastStarknetContractArtifacts;
use sncast::{
    extract_or_generate_salt,
    helpers::{configuration::CastConfig, rpc::RpcArgs},
    response::{
        errors::StarknetCommandError, ui::UI, utils::contract_address::ContractAddressResponse,
    },
    udc_uniqueness,
};
use starknet_rust::core::utils::{get_selector_from_name, get_udc_deployed_address};
use starknet_types_core::felt::Felt;
use std::collections::HashMap;

use crate::calldata_to_felts;
use crate::starknet_commands::deploy::DeployCommonArgs;
use crate::starknet_commands::utils::class_hash::sierra_class_from_artifacts;
use crate::starknet_commands::utils::serialize::{Location, resolve_abi};

#[derive(Args, Debug)]
#[command(about = "Calculate the address of a not yet deployed contract")]
pub struct ContractAddress {
    #[command(flatten)]
    pub common: DeployCommonArgs,

    /// Deployer account address, required when --unique is set
    #[arg(long)]
    pub account_address: Option<Felt>,

    #[command(flatten)]
    pub rpc: Option<RpcArgs>,
}

pub async fn get_contract_address(
    args: ContractAddress,
    artifacts: Option<HashMap<String, CastStarknetContractArtifacts>>,
    config: CastConfig,
    ui: &UI,
) -> Result<ContractAddressResponse, StarknetCommandError> {
    if args.common.unique && args.account_address.is_none() {
        return Err(anyhow::anyhow!("--account-address is required when --unique is set").into());
    }

    let salt = extract_or_generate_salt(args.common.salt);
    let selector =
        get_selector_from_name("constructor").context("Failed to get constructor selector")?;

    let (class_hash, abi) = if let Some(class_hash) = args.common.contract_identifier.class_hash {
        let abi = if args.common.arguments.arguments.is_some() {
            resolve_abi(Location::ClassHash(class_hash), args.rpc, &config, ui).await?
        } else {
            vec![]
        };
        (class_hash, abi)
    } else {
        let contract_name = args.common.contract_identifier.contract_name.unwrap();
        let sierra = sierra_class_from_artifacts(
            &contract_name,
            artifacts
                .as_ref()
                .expect("artifacts must be provided when --contract-name is used"),
        )?;
        (
            sierra.class_hash().map_err(anyhow::Error::from)?,
            sierra.abi,
        )
    };

    let deploy_args = args.common.arguments;
    let calldata = if let Some(raw) = deploy_args.constructor_calldata {
        calldata_to_felts(&raw)?
    } else if let Some(ref arguments_str) = deploy_args.arguments {
        transform(arguments_str, &abi, &selector)?
    } else {
        vec![]
    };

    let account_address = args.account_address.unwrap_or(Felt::ZERO);
    let contract_address = get_udc_deployed_address(
        salt,
        class_hash,
        &udc_uniqueness(args.common.unique, account_address),
        &calldata,
    );

    Ok(ContractAddressResponse {
        contract_address: contract_address.into_(),
    })
}
