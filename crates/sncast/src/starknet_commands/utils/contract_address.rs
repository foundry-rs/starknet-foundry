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
use starknet_rust::core::types::contract::AbiEntry;
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

    /// Deployer account address, used to modify the salt when --unique is set. Defaults to zero.
    #[arg(long, default_value_t = Felt::ZERO)]
    pub account_address: Felt,

    #[command(flatten)]
    pub rpc: Option<RpcArgs>,
}

pub async fn get_contract_address(
    args: ContractAddress,
    artifacts: Option<HashMap<String, CastStarknetContractArtifacts>>,
    config: CastConfig,
    ui: &UI,
) -> Result<ContractAddressResponse, StarknetCommandError> {
    let salt = extract_or_generate_salt(args.common.salt);
    let (class_hash, abi) = if let Some(class_hash) = args.common.contract_identifier.class_hash {
        let abi = if args.common.arguments.arguments.is_some() {
            resolve_abi(Location::ClassHash(class_hash), args.rpc, &config, ui).await?
        } else {
            vec![]
        };
        (class_hash, abi)
    } else {
        let contract_name = args
            .common
            .contract_identifier
            .contract_name
            .expect("contract_name must be set when class_hash is None");
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
        let selector = get_selector_from_name("constructor")
            .context("Failed to get constructor selector")?;
        transform(arguments_str, &abi, &selector)?
    } else {
        vec![]
    };

    if !calldata.is_empty() && !abi.is_empty() {
        let has_constructor = abi.iter().any(|e| matches!(e, AbiEntry::Constructor(_)));
        if !has_constructor {
            return Err(
                anyhow::anyhow!("Calldata provided but the contract has no constructor").into(),
            );
        }
    }

    let contract_address = get_udc_deployed_address(
        salt,
        class_hash,
        &udc_uniqueness(args.common.unique, args.account_address),
        &calldata,
    );

    Ok(ContractAddressResponse {
        contract_address: contract_address.into_(),
    })
}
