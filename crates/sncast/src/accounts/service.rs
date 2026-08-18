use anyhow::{Context, Result, ensure};
use starknet_rust::accounts::SingleOwnerAccount;
use starknet_rust::core::types::{BlockId, BlockTag};
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_rust::signers::{LocalWallet, Signer, SigningKey};
use starknet_types_core::felt::Felt;
use url::Url;

use crate::accounts::{AccountRecord, AccountRepository, AccountSelector};
use crate::compat::starkli;
use crate::helpers::account::get_account_from_devnet;
use crate::response::ui::UI;
use crate::signers::{RuntimeSigner, SignerKind, SignerProviderContext, SignerResolver};
use crate::{RuntimeAccount, chain_id_to_network_name, get_chain_id};

#[derive(Default)]
pub struct AccountService {
    repository: AccountRepository,
    signer_resolver: SignerResolver,
}

impl AccountService {
    pub async fn connected_account<'a>(
        &self,
        selector: &AccountSelector,
        provider: &'a JsonRpcClient<HttpTransport>,
        devnet_url: Option<&Url>,
        ui: &UI,
    ) -> Result<RuntimeAccount<'a>> {
        match selector {
            AccountSelector::Named {
                name,
                accounts_file,
            } => {
                let chain_id = get_chain_id(provider).await?;
                let network = chain_id_to_network_name(chain_id);
                let account = self
                    .repository
                    .find(accounts_file, &network, name.as_str())
                    .map_err(anyhow::Error::from)?;
                let context = SignerProviderContext { accounts_file, ui };
                let signer = self
                    .signer_resolver
                    .resolve_and_verify(&account.signer, account.public_key, &context)
                    .await?;
                Self::build_runtime_account(account, chain_id, provider, signer).await
            }
            AccountSelector::LegacyStarkli {
                account_file,
                keystore_file,
            } => {
                let chain_id = get_chain_id(provider).await?;
                let account = starkli::load_account(account_file.as_str(), keystore_file)?;
                let private_key = account
                    .signer
                    .private_key()
                    .context("Private key not found in starkli account")?;
                let signer = RuntimeSigner::from_starknet_signer(
                    LocalWallet::from_signing_key(SigningKey::from_secret_scalar(private_key)),
                    SignerKind::Keystore,
                );
                verify_public_key(&account, &signer).await?;
                Self::build_runtime_account(account, chain_id, provider, signer).await
            }
            AccountSelector::Devnet { index } => {
                let url = devnet_url.context("Devnet account requires a devnet URL")?;
                get_account_from_devnet(*index, provider, url).await
            }
        }
    }

    pub(crate) async fn build_runtime_account<'a>(
        account: AccountRecord,
        chain_id: Felt,
        provider: &'a JsonRpcClient<HttpTransport>,
        signer: RuntimeSigner,
    ) -> Result<SingleOwnerAccount<&'a JsonRpcClient<HttpTransport>, RuntimeSigner>> {
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

async fn verify_public_key(account: &AccountRecord, signer: &RuntimeSigner) -> Result<()> {
    let actual = signer.get_public_key().await?.scalar();
    ensure!(
        actual == account.public_key,
        "{} signer public key does not match the account: expected {:#x}, got {actual:#x}",
        signer.kind(),
        account.public_key
    );
    Ok(())
}
