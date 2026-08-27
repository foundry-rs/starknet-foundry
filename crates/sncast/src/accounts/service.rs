use anyhow::Context;
use starknet_rust::accounts::SingleOwnerAccount;
use starknet_rust::core::types::{BlockId, BlockTag};
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_rust::signers::Signer;
use starknet_types_core::felt::Felt;
use url::Url;

use crate::RuntimeAccount;
use crate::accounts::{AccountRecord, AccountRepository, AccountSelector};
use crate::helpers::account::get_account_from_devnet;
use crate::response::ui::UI;
use crate::signers::spec::PrivateKeySource;
use crate::signers::{RuntimeSigner, SignerError};

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
        chain_id: Felt,
        provider: &'a JsonRpcClient<HttpTransport>,
        devnet_url: Option<&Url>,
        ui: &UI,
    ) -> anyhow::Result<RuntimeAccount<'a>> {
        match selector {
            AccountSelector::Named { name } => {
                let account = crate::get_account_record_from_repository(
                    name.as_str(),
                    chain_id,
                    &self.repository,
                )?;
                let signer = RuntimeSigner::from_spec(account.signer.clone(), ui).await?;
                verify_public_key(&account, &signer).await?;
                Self::build_runtime_account(account, chain_id, provider, signer).await
            }
            AccountSelector::LegacyStarkli {
                account_file,
                keystore_file,
            } => {
                let account: AccountRecord =
                    crate::get_account_data_from_keystore(account_file.as_str(), keystore_file)?
                        .try_into()?;
                let private_key = account
                    .signer
                    .private_key()
                    .context("Private key not found in starkli account")?;
                let signer =
                    RuntimeSigner::from_private_key(private_key, PrivateKeySource::Keystore);
                verify_public_key(&account, &signer).await?;
                Self::build_runtime_account(account, chain_id, provider, signer).await
            }
            AccountSelector::Devnet { index } => {
                let url = devnet_url.context("Devnet account requires a devnet URL")?;
                get_account_from_devnet(*index, chain_id, provider, url).await
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

async fn verify_public_key(
    account: &AccountRecord,
    signer: &RuntimeSigner,
) -> Result<(), SignerError> {
    let expected = account.public_key;
    let actual = signer.get_public_key().await?.scalar();
    if expected != actual {
        Err(SignerError::PublicKeyMismatch {
            kind: signer.kind(),
            expected,
            actual,
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts::{AccountName, ChainId};
    use crate::helpers::configuration::CastConfig;
    use crate::helpers::rpc::RpcArgs;
    use crate::signers::{KeystoreSpec, PrivateKeySpec, SignerSpec};
    use camino::Utf8PathBuf;
    use serde_json::json;
    use starknet_rust::accounts::Account;
    use starknet_rust::signers::SigningKey;
    use test_case::test_case;
    use wiremock::matchers::body_partial_json;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test_case(false, false; "matching private key")]
    #[test_case(false, true; "wrong private key")]
    #[test_case(true, false; "matching keystore")]
    #[test_case(true, true; "substituted keystore")]
    #[tokio::test]
    async fn named_accounts_verify_signer_identity(keystore: bool, wrong_key: bool) {
        let directory = tempfile::tempdir().unwrap();
        let path = Utf8PathBuf::from_path_buf(directory.path().join("accounts.json")).unwrap();
        let repository = AccountRepository::try_new(&path).unwrap();
        let key = if wrong_key { Felt::TWO } else { Felt::ONE };
        let signer = if keystore {
            let path = Utf8PathBuf::from_path_buf(directory.path().join("key.json")).unwrap();
            SigningKey::from_secret_scalar(key)
                .save_as_keystore(&path, &std::env::var("PATH").unwrap())
                .unwrap();
            SignerSpec::Keystore(KeystoreSpec::new(path, Some("PATH".to_owned())))
        } else {
            SignerSpec::PrivateKey(PrivateKeySpec::new(key))
        };
        repository
            .insert(
                ChainId::from(crate::SEPOLIA),
                AccountName::new("alice").unwrap(),
                AccountRecord {
                    address: Felt::THREE,
                    public_key: SigningKey::from_secret_scalar(Felt::ONE)
                        .verifying_key()
                        .scalar(),
                    signer,
                    legacy: Some(false),
                    deployed: Some(true),
                    class_hash: None,
                    salt: None,
                    account_type: None,
                },
            )
            .unwrap();
        let original = std::fs::read(&path).unwrap();
        let server = MockServer::start().await;
        for (method, result) in [
            ("starknet_chainId", format!("{:#x}", crate::SEPOLIA)),
            ("starknet_getNonce", "0x0".to_owned()),
        ] {
            Mock::given(body_partial_json(json!({"method": method})))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "jsonrpc": "2.0", "id": 1, "result": result
                })))
                .mount(&server)
                .await;
        }
        let provider = crate::get_provider(&server.uri().parse().unwrap()).unwrap();
        let config = CastConfig {
            account: "alice".to_owned(),
            accounts_file: path.clone(),
            ..Default::default()
        };
        let result =
            crate::get_account(&config, &provider, &RpcArgs::default(), &UI::default()).await;
        if wrong_key {
            assert!(matches!(
                result.unwrap_err().downcast_ref::<SignerError>(),
                Some(SignerError::PublicKeyMismatch { .. })
            ));
        } else {
            assert_eq!(result.unwrap().address(), Felt::THREE);
        }
        assert_eq!(std::fs::read(path).unwrap(), original);
    }
}
