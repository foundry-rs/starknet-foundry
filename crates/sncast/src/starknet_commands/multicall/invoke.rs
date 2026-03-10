use anyhow::Result;
use clap::Args;
use starknet_rust::core::{types::Call, utils::get_selector_from_name};

use crate::{
    Arguments, calldata_to_felts,
    starknet_commands::{
        invoke::InvokeCommonArgs,
        multicall::{
            contract_registry::ContractRegistry,
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
        let calldata = parse_inputs(item.inputs(), contracts)?;
        let contract_address =
            if let Some(addr) = contracts.get_address_by_id(item.contract_address()) {
                addr
            } else {
                item.contract_address().parse()?
            };
        let invoke = MulticallInvoke {
            common: InvokeCommonArgs {
                contract_address,
                function: item.function().clone(),
                arguments: Arguments {
                    calldata: Some(
                        calldata
                            .iter()
                            .map(std::string::ToString::to_string)
                            .collect(),
                    ),
                    arguments: None,
                },
            },
        };

        Ok(invoke)
    }
    pub async fn build_call(&self, contract_registry: &mut ContractRegistry) -> Result<Call> {
        let selector = get_selector_from_name(&self.common.function)?;
        let arguments = replaced_calldata(&self.common.arguments, contract_registry)?;

        let calldata = if let Some(raw_calldata) = &arguments.calldata {
            calldata_to_felts(raw_calldata)?
        } else {
            let class_hash = contract_registry
                .get_class_hash_by_address(&self.common.contract_address)
                .await?;
            let contract_class = contract_registry
                .get_contract_class_by_class_hash(&class_hash)
                .await?;
            arguments.try_into_calldata(contract_class, &selector)?
        };

        Ok(Call {
            to: self.common.contract_address,
            selector,
            calldata,
        })
    }
}

pub fn replaced_calldata(
    function_arguments: &Arguments,
    contract_registry: &ContractRegistry,
) -> Result<Arguments> {
    Ok(
        match (&function_arguments.calldata, &function_arguments.arguments) {
            (Some(calldata), None) => {
                let replaced_calldata = calldata
                    .iter()
                    .map(|input| {
                        if let Some(address) = contract_registry.get_address_by_id(input) {
                            Ok(address.to_string())
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
            (Some(_), Some(_)) => anyhow::bail!(
                "Invalid arguments: both `calldata` and `arguments` are set. Please provide only one."
            ),
        },
    )
}
