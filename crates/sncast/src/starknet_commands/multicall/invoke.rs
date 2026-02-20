use anyhow::Result;
use clap::Args;
use starknet_rust::core::{types::Call, utils::get_selector_from_name};

use crate::{
    Arguments,
    starknet_commands::{invoke::InvokeCommonArgs, multicall::ctx::MulticallCtx},
};

#[derive(Args)]
pub(crate) struct MulticallInvoke {
    /// Optional identifier to reference this step in later steps
    #[arg(long)]
    pub id: Option<String>,

    #[command(flatten)]
    pub common: InvokeCommonArgs,
}

impl MulticallInvoke {
    pub(crate) async fn to_call(&self, ctx: &mut MulticallCtx) -> Result<Call> {
        let selector = get_selector_from_name(&self.common.function)?;
        let class_hash = ctx
            .cache
            .get_class_hash_by_address(&self.common.contract_address)
            .await?;
        let contract_class = ctx
            .cache
            .get_contract_class_by_class_hash(&class_hash)
            .await?;
        let arguments = replaced_calldata(self.common.arguments.clone(), ctx)?;
        let calldata = arguments.try_into_calldata(contract_class, &selector)?;

        Ok(Call {
            to: self.common.contract_address,
            selector,
            calldata,
        })
    }
}

// pub(crate) async fn process_invoke(
//     invoke: MulticallInvoke,
//     ctx: &mut MulticallCtx,
// ) -> Result<Call> {
// let selector = get_selector_from_name(&invoke.common.function)?;
// let class_hash = ctx
//     .cache
//     .get_class_hash_by_address(&invoke.common.contract_address)
//     .await?;
// let contract_class = ctx
//     .cache
//     .get_contract_class_by_class_hash(&class_hash)
//     .await?;
// let arguments = replaced_calldata(invoke.common.arguments.clone(), ctx)?;
// let calldata = arguments.try_into_calldata(contract_class, &selector)?;

// Ok(Call {
//     to: invoke.common.contract_address,
//     selector,
//     calldata,
// })
// }

pub(crate) fn replaced_calldata(
    function_arguments: Arguments,
    ctx: &MulticallCtx,
) -> Result<Arguments> {
    Ok(
        match (&function_arguments.calldata, &function_arguments.arguments) {
            (Some(calldata), None) => {
                let replaced_calldata = calldata
                    .iter()
                    .map(|input| {
                        if let Some(address) = ctx.get_address_by_id(input) {
                            Ok(address.to_string())
                        } else {
                            Ok(input.to_string())
                        }
                    })
                    .collect::<Result<Vec<String>>>()?;
                Arguments {
                    calldata: Some(replaced_calldata),
                    arguments: None,
                }
            }
            (None, Some(_)) | (None, None) => function_arguments,
            (Some(_), Some(_)) => unreachable!(),
        },
    )
}
