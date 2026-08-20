use anyhow::{Context, Result};
use starknet_rust::accounts::SingleOwnerAccount;
use starknet_rust::core::types::{BlockId, BlockTag};
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_types_core::felt::Felt;
use url::Url;

use crate::accounts::{AccountRecord, AccountRepository, AccountSelector};
use crate::helpers::account::get_account_from_devnet;
use crate::response::ui::UI;
use crate::signers::{RuntimeSigner, SignerProviderContext, SignerResolver};
use crate::{RuntimeAccount, chain_id_to_network_name, get_chain_id};

pub struct AccountService {
    repository: AccountRepository,
    signer_resolver: SignerResolver,
}

impl AccountService {
    #[must_use]
    pub fn new(repository: AccountRepository) -> Self {
        Self {
            repository,
            signer_resolver: SignerResolver::default(),
        }
    }

    pub async fn connected_account<'a>(
        &self,
        selector: &AccountSelector,
        provider: &'a JsonRpcClient<HttpTransport>,
        devnet_url: Option<&Url>,
        ui: &UI,
    ) -> Result<RuntimeAccount<'a>> {
        match selector {
            AccountSelector::Named { name } => {
                let chain_id = get_chain_id(provider).await?;
                let network = chain_id_to_network_name(chain_id);
                let account = self
                    .repository
                    .find(&network, name.as_str())
                    .map_err(anyhow::Error::from)?;
                let context = SignerProviderContext {
                    repository: &self.repository,
                    ui,
                };
                let signer = self
                    .signer_resolver
                    .resolve_and_verify(&account.signer, account.public_key, &context)
                    .await?;
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
    ) -> Result<SingleOwnerAccount<&JsonRpcClient<HttpTransport>, RuntimeSigner>> {
        let connected = account.as_connected()?;
        let address = connected.address();
        crate::verify_account_address(address, chain_id, provider).await?;
        let encoding =
            crate::get_account_encoding(account.legacy, account.class_hash, address, provider)
                .await?;

        let mut runtime = SingleOwnerAccount::new(provider, signer, address, chain_id, encoding);
        runtime.set_block_id(BlockId::Tag(BlockTag::PreConfirmed));
        Ok(runtime)
    }
}
