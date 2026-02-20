use anyhow::{Context, Result};
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

        let is_id = self.common.contract_address.starts_with('@');
        let contract_address = if is_id {
            let id = self.common.contract_address.trim_start_matches('@');
            ctx.get_address_by_id(id)
                .with_context(|| format!("No contract address found for id: {id}. Ensure the referenced id is defined in a previous step."))?
        } else {
            self.common.contract_address.parse()?
        };

        // let contract_address = if let Some(address) =
        //     ctx.get_address_by_id(&self.common.contract_address.replace("@", ""))
        // {
        //     address
        // } else {
        //     self.common.contract_address.parse()?
        // };
        let class_hash = ctx
            .cache
            .get_class_hash_by_address(&contract_address)
            .await?;
        let contract_class = ctx
            .cache
            .get_contract_class_by_class_hash(&class_hash)
            .await?;
        let arguments = replaced_calldata(self.common.arguments.clone(), ctx)?;
        let calldata = arguments.try_into_calldata(contract_class, &selector)?;

        Ok(Call {
            to: contract_address,
            selector,
            calldata,
        })
    }
}

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
