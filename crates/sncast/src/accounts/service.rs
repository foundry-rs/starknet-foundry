use anyhow::Context;
use starknet_rust::accounts::SingleOwnerAccount;
use starknet_rust::core::types::{BlockId, BlockTag};
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_types_core::felt::Felt;
use url::Url;

use crate::accounts::{AccountRecord, AccountRepository, AccountSelector};
use crate::helpers::account::get_account_from_devnet;
use crate::response::ui::UI;
use crate::signers::RuntimeSigner;
use crate::{RuntimeAccount, get_chain_id};

pub struct AccountService {
    repository: AccountRepository,
}

impl AccountService {
    #[must_use]
    pub fn new(repository: AccountRepository) -> Self {
        Self { repository }
    }

    pub async fn connected_account<'a>(
        &self,
        selector: &AccountSelector,
        provider: &'a JsonRpcClient<HttpTransport>,
        devnet_url: Option<&Url>,
        ui: &UI,
    ) -> anyhow::Result<RuntimeAccount<'a>> {
        match selector {
            AccountSelector::Named { name } => {
                let chain_id = get_chain_id(provider).await?;
                let account = crate::get_account_record_from_repository(
                    name.as_str(),
                    chain_id,
                    &self.repository,
                )?;
                let signer = RuntimeSigner::from_spec(account.signer.clone(), ui).await?;
                Self::build_runtime_account(account, chain_id, provider, signer).await
            }
            AccountSelector::Devnet { index } => {
                let url = devnet_url.context("Devnet account requires a devnet URL")?;
                get_account_from_devnet(*index, provider, url).await
            }
        }
    }

    pub(crate) async fn build_runtime_account(
        account: AccountRecord,
        chain_id: Felt,
        provider: &JsonRpcClient<HttpTransport>,
        signer: RuntimeSigner,
    ) -> anyhow::Result<RuntimeAccount<'_>> {
        let address = account.address;
        crate::verify_account_address(address, chain_id, provider).await?;
        let encoding =
            crate::get_account_encoding(account.legacy, account.class_hash, address, provider)
                .await?;

        let mut runtime = SingleOwnerAccount::new(provider, signer, address, chain_id, encoding);
        runtime.set_block_id(BlockId::Tag(BlockTag::PreConfirmed));
        Ok(runtime)
    }
}
