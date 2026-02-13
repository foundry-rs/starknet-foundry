use crate::{
    CommonInvokeArgs,
    starknet_commands::{deploy::CommonDeployArgs, multicall::ctx::MulticallCtx},
};
use anyhow::{Result, bail};
use clap::Args;
use sncast::{
    extract_or_generate_salt, get_contract_class, helpers::constants::UDC_ADDRESS, udc_uniqueness,
};
use starknet_rust::{
    core::{
        types::Call,
        utils::{get_selector_from_name, get_udc_deployed_address},
    },
    providers::{JsonRpcClient, Provider, jsonrpc::HttpTransport},
};
use starknet_types_core::felt::Felt;

#[derive(Args, Debug, Clone)]
pub struct MulticallDeploy {
    /// Optional identifier to reference this step in later steps
    #[arg(long)]
    pub id: Option<String>,

    #[command(flatten)]
    pub inner: CommonDeployArgs,
}

pub async fn deploy(
    deploy: MulticallDeploy,
    ctx: &mut MulticallCtx,
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<()> {
    let salt = extract_or_generate_salt(deploy.inner.salt);

    let class_hash = if let Some(class_hash) = deploy.inner.contract_identifier.class_hash.clone() {
        class_hash
    } else {
        bail!("multicall does not support deployment with `--name` flag")
    };

    let arguments: CommonInvokeArgs = deploy.inner.arguments.clone().into();
    let contract_class = get_contract_class(class_hash, &provider).await?;

    let selector = get_selector_from_name("constructor").unwrap();
    let constructor_calldata = arguments.try_into_calldata(contract_class, &selector)?;

    let mut calldata = vec![
        class_hash,
        salt,
        Felt::from(u8::from(deploy.inner.unique)),
        constructor_calldata.len().into(),
    ];

    calldata.extend(&constructor_calldata);

    ctx.add_call(Call {
        to: UDC_ADDRESS,
        selector: get_selector_from_name("deployContract")?,
        calldata,
    });

    let contract_address = get_udc_deployed_address(
        salt,
        class_hash,
        &udc_uniqueness(deploy.inner.unique, provider.chain_id().await?),
        &constructor_calldata,
    );
    ctx.insert_address_class_hash_mapping(contract_address, class_hash)?;

    if let Some(id) = &deploy.id {
        ctx.insert_id_to_address(id.clone(), contract_address)?;
    }

    Ok(())
}
