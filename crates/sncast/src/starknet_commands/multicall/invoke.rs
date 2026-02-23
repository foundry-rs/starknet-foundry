use anyhow::{Context, Result};
use clap::Args;
use starknet_rust::core::{types::Call, utils::get_selector_from_name};

use crate::{
    Arguments, calldata_to_felts,
    starknet_commands::{
        invoke::InvokeCommonArgs, multicall::contracts_registry::ContractsRegistry,
    },
};

#[derive(Args)]
pub(crate) struct MulticallInvoke {
    #[command(flatten)]
    pub common: InvokeCommonArgs,
}

impl MulticallInvoke {
    pub(crate) async fn build_call(
        &self,
        contracts_registry: &mut ContractsRegistry,
    ) -> Result<Call> {
        let selector = get_selector_from_name(&self.common.function)?;
        let is_id = self.common.contract_address.starts_with('@');
        let contract_address = if is_id {
            let id = self.common.contract_address.trim_start_matches('@');
            contracts_registry.get_address_by_id(id)
                .with_context(|| format!("No contract address found for id: {id}. Ensure the referenced id is defined in a previous step."))?
        } else {
            self.common
                .contract_address
                .parse()
                .with_context(|| {
                    format!(
                        "Failed to parse contract address `{}`. Expected a hexadecimal Starknet address like `0x123...`. \
If you intended to reference an address from a previous step, use `@<id>` instead (for example, `@deployed_address`).",
                        self.common.contract_address
                    )
                })?
        };
        let arguments = replaced_calldata(&self.common.arguments, contracts_registry)?;

        let calldata = if let Some(raw_calldata) = &arguments.calldata {
            calldata_to_felts(raw_calldata)?
        } else {
            let class_hash = contracts_registry
                .cache
                .get_class_hash_by_address(&contract_address)
                .await?;
            let contract_class = contracts_registry
                .cache
                .get_contract_class_by_class_hash(&class_hash)
                .await?;
            arguments.try_into_calldata(&contract_class, &selector)?
        };

        Ok(Call {
            to: contract_address,
            selector,
            calldata,
        })
    }
}

pub(crate) fn replaced_calldata(
    function_arguments: &Arguments,
    contracts_registry: &ContractsRegistry,
) -> Result<Arguments> {
    Ok(
        match (&function_arguments.calldata, &function_arguments.arguments) {
            (Some(calldata), None) => {
                let replaced_calldata = calldata
                    .iter()
                    .map(|input| {
                        let is_id = input.starts_with('@');
                        if is_id {
                            let id = input.trim_start_matches('@');
                            if let Some(address) = contracts_registry.get_address_by_id(id) {
                                Ok(address.to_string())
                            } else {
                                anyhow::bail!("No contract address found for id: {id}. Ensure the referenced id is defined in a previous step.")
                            }
                        } else {
                            Ok(input.clone())
                        }
                    })
                    .collect::<Result<Vec<String>>>()?;
                Arguments {
                    calldata: Some(replaced_calldata),
                    arguments: None,
                }
            }
            (None, _) => function_arguments.clone(),
            (Some(_), Some(_)) => unreachable!(
                "Clap should ensure that only one of calldata or arguments is provided"
            ),
        },
    )
}
