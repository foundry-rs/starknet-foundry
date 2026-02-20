use anyhow::Result;
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
use crate::starknet_commands::multicall::ctx::MulticallCtx;
use crate::starknet_commands::multicall::invoke::replaced_calldata;

#[derive(Args)]
pub struct MulticallDeploy {
    /// Optional identifier to reference this step in later steps
    #[arg(long)]
    pub id: Option<String>,

    #[command(flatten)]
    pub common: DeployCommonArgs,
}

impl MulticallDeploy {
    pub(crate) async fn to_call(
        &self,
        account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
        ctx: &mut MulticallCtx,
    ) -> Result<Call> {
        let salt = extract_or_generate_salt(self.common.salt);
        let class_hash = self
            .common
            .contract_identifier
            .class_hash
            .expect("Class hash must be provided for deploy calls");

        let contract_class = ctx
            .cache
            .get_contract_class_by_class_hash(&class_hash)
            .await?;
        let selector = get_selector_from_name("constructor")?;
        let constructor_arguments = replaced_calldata(self.common.arguments.clone(), ctx)?;
        let constructor_calldata =
            constructor_arguments.try_into_calldata(contract_class, &selector)?;

        let mut calldata = vec![
            class_hash,
            salt,
            Felt::from(u8::from(self.common.unique)),
            constructor_calldata.len().into(),
        ];

        calldata.extend(constructor_calldata.clone());

        let contract_address = get_udc_deployed_address(
            salt,
            class_hash,
            &udc_uniqueness(self.common.unique, account.address()),
            &constructor_calldata,
        );

        if ctx
            .cache
            .get_class_hash_by_address_local(&contract_address)
            .is_none()
        {
            ctx.cache
                .try_insert_address_to_class_hash(contract_address, class_hash)?;
        }

        // Store the contract address in the context with the provided id for later use in invoke calls
        if let Some(id) = &self.id {
            ctx.try_insert_id_to_address(id.to_string(), contract_address)?;
        }

        Ok(Call {
            to: UDC_ADDRESS,
            selector: get_selector_from_name("deployContract")?,
            calldata,
        })
    }
}
