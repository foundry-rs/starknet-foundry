use anyhow::Result;
use clap::Args;
use sncast::helpers::configuration::CastConfig;
use starknet_rust::core::{types::Call, utils::get_selector_from_name};

use crate::{
    Arguments, abi_from_contract_class, calldata_to_felts,
    starknet_commands::{
        invoke::InvokeCommonArgs,
        multicall::{
            contract_registry::ContractRegistry,
            run::{InvokeItem, parse_inputs},
        },
        utils::felt_or_id::{ContractAddress, resolve_multicall_calldata_to_felts},
    },
};

#[derive(Args)]
pub struct MulticallInvoke {
    #[command(flatten)]
    pub common: InvokeCommonArgs,
}

impl MulticallInvoke {
    pub fn new_from_item(
        item: &InvokeItem,
        contracts: &ContractRegistry,
        config: &CastConfig,
    ) -> Result<Self> {
        let calldata = parse_inputs(&item.inputs, contracts, config)?;
        let invoke = MulticallInvoke {
            common: InvokeCommonArgs {
                contract_address: ContractAddress(item.contract_address.clone()),
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
        config: &CastConfig,
    ) -> Result<Call> {
        let selector = get_selector_from_name(&self.common.function)?;
        let arguments = &self.common.arguments;
        let contract_address = self
            .common
            .contract_address
            .resolve_in_multicall(contract_registry, config)?;

        let calldata = if let Some(raw_calldata) = &arguments.calldata {
            resolve_multicall_calldata_to_felts(raw_calldata, config, contract_registry)?
        } else {
            let class_hash = contract_registry
                .get_class_hash_by_address(&contract_address)
                .await?;
            let contract_class = contract_registry
                .get_contract_class_by_class_hash(&class_hash)
                .await?;
            arguments
                .clone()
                .try_into_calldata(&abi_from_contract_class(contract_class)?, &selector, config)?
        };

        Ok(Call {
            to: contract_address,
            selector,
            calldata,
        })
    }
}
