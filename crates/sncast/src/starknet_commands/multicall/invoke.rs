use anyhow::{Context, Result};
use clap::Args;
use starknet_rust::core::{types::Call, utils::get_selector_from_name};

use crate::{
    Arguments, calldata_to_felts,
    starknet_commands::{
        invoke::InvokeCommonArgs,
        multicall::{
            contract_registry::ContractRegistry,
            mode::MulticallMode,
            replaced_arguments,
            run::{InvokeItem, parse_inputs},
        },
    },
};

#[derive(Args)]
pub struct MulticallInvoke {
    #[command(flatten)]
    pub common: InvokeCommonArgs,
}

impl MulticallInvoke {
    pub fn new_from_item(item: &InvokeItem, contracts: &ContractRegistry) -> Result<Self> {
        let calldata = parse_inputs(&item.inputs, contracts)?;
        let contract_address =
            if let Some(addr) = contracts.get_address_by_id(&item.contract_address) {
                addr
            } else {
                item.contract_address.parse()?
            };
        let invoke = MulticallInvoke {
            common: InvokeCommonArgs {
                contract_address: contract_address.to_string(),
                function: item.function.clone(),
                arguments: Arguments {
                    calldata: Some(calldata.iter().map(ToString::to_string).collect()),
                    arguments: None,
                },
            },
        };

        Ok(invoke)
    }
    pub async fn build_call(
        &self,
        contract_registry: &mut ContractRegistry,
        mode: MulticallMode,
    ) -> Result<Call> {
        let selector = get_selector_from_name(&self.common.function)?;
        let contract_address = if let Some(id_key) = mode.id_key(&self.common.contract_address) {
            contract_registry
                .get_address_by_id(id_key)
                .with_context(|| {
                    format!(
                        "No contract address found for id: {id_key}. Ensure the referenced id is defined in a previous step."
                    )
                })?
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
        let arguments = replaced_arguments(&self.common.arguments, contract_registry, mode)?;

        let calldata = if let Some(raw_calldata) = &arguments.calldata {
            calldata_to_felts(raw_calldata)?
        } else {
            let class_hash = contract_registry
                .get_class_hash_by_address(&contract_address)
                .await?;
            let contract_class = contract_registry
                .get_contract_class_by_class_hash(&class_hash)
                .await?;
            arguments.try_into_calldata(contract_class, &selector)?
        };

        Ok(Call {
            to: contract_address,
            selector,
            calldata,
        })
    }
}
