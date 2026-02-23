use anyhow::{Context, Result};
use clap::Args;
use sncast::helpers::constants::UDC_ADDRESS;
use sncast::{extract_or_generate_salt, udc_uniqueness};
use starknet_rust::accounts::{Account, SingleOwnerAccount};
use starknet_rust::core::types::Call;
use starknet_rust::core::utils::{get_selector_from_name, get_udc_deployed_address};
use starknet_rust::providers::JsonRpcClient;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::signers::LocalWallet;
use starknet_types_core::felt::Felt;

use crate::starknet_commands::deploy::DeployCommonArgs;
use crate::starknet_commands::multicall::contracts_registry::ContractsRegistry;
use crate::starknet_commands::multicall::invoke::replaced_calldata;
use crate::{Arguments, calldata_to_felts};

#[derive(Args)]
pub(crate) struct MulticallDeploy {
    /// Optional identifier to reference this step in later steps
    #[arg(long)]
    pub id: Option<String>,

    #[command(flatten)]
    pub common: DeployCommonArgs,
}

impl MulticallDeploy {
    pub(crate) async fn convert_to_call(
        &self,
        account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
        contracts_registry: &mut ContractsRegistry,
    ) -> Result<Call> {
        let salt = extract_or_generate_salt(self.common.salt);
        let constructor_arguments = replaced_calldata(
            &Arguments::from(self.common.arguments.clone()),
            contracts_registry,
        )?;

        let constructor_selector = get_selector_from_name("constructor")?;
        let class_hash = self
            .common
            .contract_identifier
            .class_hash
            .context("Using deploy with multicall requires providing `--class-hash`")?;
        let constructor_calldata =
            if let Some(raw_calldata) = &self.common.arguments.constructor_calldata {
                calldata_to_felts(raw_calldata)?
            } else {
                let contract_class = contracts_registry
                    .cache
                    .get_contract_class_by_class_hash(&class_hash)
                    .await?;
                constructor_arguments.try_into_calldata(&contract_class, &constructor_selector)?
            };

        let mut calldata = vec![
            class_hash,
            salt,
            Felt::from(u8::from(self.common.unique)),
            constructor_calldata.len().into(),
        ];
        calldata.extend_from_slice(&constructor_calldata);

        let contract_address = get_udc_deployed_address(
            salt,
            class_hash,
            &udc_uniqueness(self.common.unique, account.address()),
            &constructor_calldata,
        );

        if contracts_registry
            .cache
            .get_class_hash_by_address_local(&contract_address)
            .is_none()
        {
            contracts_registry
                .cache
                .insert_new_address(contract_address, class_hash)?;
        }

        // Store the contract address in the context with the provided id for later use in invoke calls
        if let Some(id) = &self.id {
            contracts_registry.insert_new_id_to_address(id.clone(), contract_address)?;
        }

        Ok(Call {
            to: UDC_ADDRESS,
            selector: get_selector_from_name("deployContract")?,
            calldata,
        })
    }
}
