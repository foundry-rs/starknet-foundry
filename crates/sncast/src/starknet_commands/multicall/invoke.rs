use crate::starknet_commands::{invoke::InvokeArgs, multicall::ctx::MulticallCtx};
use anyhow::{Context, Result};
use sncast::{get_class_hash_by_address, get_contract_class};
use starknet_rust::{
    core::{types::Call, utils::get_selector_from_name},
    providers::{JsonRpcClient, jsonrpc::HttpTransport},
};
use starknet_types_core::felt::Felt;

pub async fn invoke(
    invoke: InvokeArgs,
    ctx: &mut MulticallCtx,
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<()> {
    let maybe_id = invoke.contract_address.trim_start_matches("@");
    let contract_address = if let Some(address) = ctx.get_address_by_id(maybe_id) {
        address
    } else {
        invoke.contract_address.parse::<Felt>().with_context(|| {
            format!(
                "Failed to parse contract address: {}",
                invoke.contract_address
            )
        })?
    };

    let class_hash = if let Some(class_hash) = ctx.get_class_hash_by_address(&contract_address) {
        class_hash
    } else {
        get_class_hash_by_address(&provider, contract_address).await?
    };
    let contract_class = get_contract_class(class_hash, &provider).await?;

    let selector = get_selector_from_name(&invoke.function)
        .context("Failed to convert entry point selector to FieldElement")?;
    let calldata = invoke
        .arguments
        .clone()
        .try_into_calldata(contract_class, &selector)?;

    ctx.add_call(Call {
        to: contract_address,
        selector: get_selector_from_name(&invoke.function)?,
        calldata,
    });

    Ok(())
}
