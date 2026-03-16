use anyhow::Result;
use clap::Args;
use starknet_rust::core::{types::Call, utils::get_selector_from_name};

use crate::{
    Arguments, calldata_to_felts,
    starknet_commands::{
        invoke::InvokeCommonArgs,
        multicall::{
            contract_registry::ContractRegistry,
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
        let invoke = MulticallInvoke {
            common: InvokeCommonArgs {
                contract_address: item.contract_address.clone(),
                function: item.function.clone(),
                arguments: Arguments {
                    calldata: Some(calldata.iter().map(ToString::to_string).collect()),
                    arguments: None,
                },
            },
        };

        Ok(invoke)
    }

    pub async fn build_call(&self, contract_registry: &mut ContractRegistry) -> Result<Call> {
        let selector = get_selector_from_name(&self.common.function)?;
        let arguments = replaced_arguments(&self.common.arguments, contract_registry)?;
        let contracy_address = self.common.contract_address.as_id(contract_registry)?;

        let calldata = if let Some(raw_calldata) = &arguments.calldata {
            calldata_to_felts(raw_calldata)?
        } else {
            let class_hash = contract_registry
                .get_class_hash_by_address(&contracy_address)
                .await?;
            let contract_class = contract_registry
                .get_contract_class_by_class_hash(&class_hash)
                .await?;
            arguments.try_into_calldata(contract_class, &selector)?
        };

        Ok(Call {
            to: contracy_address,
            selector,
            calldata,
        })
    }
}
