use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8PathBuf;
use clap::Args;
use serde_json::Map;
use sncast::helpers::constants::{BRAAVOS_BASE_ACCOUNT_CLASS_HASH, KEYSTORE_PASSWORD_ENV_VAR};
use sncast::response::structs::{Felt, InvokeResponse};
use starknet::accounts::{AccountFactory, OpenZeppelinAccountFactory};
use starknet::accounts::{AccountFactoryError, ArgentAccountFactory};
use starknet::core::types::BlockTag::Pending;
use starknet::core::types::{BlockId, FieldElement, StarknetError::ClassHashNotFound};
use starknet::core::utils::get_contract_address;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::ProviderError::StarknetError;
use starknet::providers::{JsonRpcClient, Provider};
use starknet::signers::{LocalWallet, SigningKey};

use sncast::helpers::braavos::BraavosAccountFactory;
use sncast::{
    chain_id_to_network_name, check_account_file_exists, get_account_data_from_accounts_file,
    get_account_data_from_keystore, get_keystore_password, handle_account_factory_error,
    handle_rpc_error, handle_wait_for_tx, AccountType, WaitForTx,
};

#[derive(Args, Debug)]
#[command(about = "Deploy an account to the Starknet")]
pub struct Deploy {
    /// Name of the account to be deployed
    #[clap(short, long)]
    pub name: Option<String>,

    /// Max fee for the transaction
    #[clap(short, long)]
    pub max_fee: Option<FieldElement>,
}

#[allow(clippy::too_many_arguments)]
pub async fn deploy(
    provider: &JsonRpcClient<HttpTransport>,
    accounts_file: Utf8PathBuf,
    deploy_args: Deploy,
    chain_id: FieldElement,
    wait_config: WaitForTx,
    account: &str,
    keystore_path: Option<Utf8PathBuf>,
) -> Result<InvokeResponse> {
    if let Some(keystore_path_) = keystore_path {
        deploy_from_keystore(
            provider,
            chain_id,
            deploy_args.max_fee,
            wait_config,
            account,
            keystore_path_,
        )
        .await
    } else {
        let account_name = deploy_args
            .name
            .ok_or_else(|| anyhow!("Required argument `--name` not provided"))?;
        check_account_file_exists(&accounts_file)?;
        deploy_from_accounts_file(
            provider,
            accounts_file,
            account_name,
            chain_id,
            deploy_args.max_fee,
            wait_config,
        )
        .await
    }
}

async fn deploy_from_keystore(
    provider: &JsonRpcClient<HttpTransport>,
    chain_id: FieldElement,
    max_fee: Option<FieldElement>,
    wait_config: WaitForTx,
    account: &str,
    keystore_path: Utf8PathBuf,
) -> Result<InvokeResponse> {
    let account_data = get_account_data_from_keystore(account, &keystore_path)?;

    let is_deployed = account_data
        .deployed
        .ok_or_else(|| anyhow!("Failed to get status key from account JSON file"))?;
    if is_deployed {
        bail!("Account already deployed");
    }

    let private_key = SigningKey::from_keystore(
        keystore_path,
        get_keystore_password(KEYSTORE_PASSWORD_ENV_VAR)?.as_str(),
    )?;
    let public_key = account_data.public_key;

    if public_key != private_key.verifying_key().scalar() {
        bail!("Public key and private key from keystore do not match");
    }

    let salt = account_data
        .salt
        .context("Failed to get salt from keystore")?;

    let account_type = account_data
        .account_type
        .context("Failed to get account type from keystore")?;
    let class_hash = account_data
        .class_hash
        .context("Failed to get class hash from keystore")?;

    let address = match account_type {
        AccountType::Argent => get_contract_address(
            salt,
            class_hash,
            &[private_key.verifying_key().scalar(), FieldElement::ZERO],
            FieldElement::ZERO,
        ),
        AccountType::Oz => get_contract_address(
            salt,
            class_hash,
            &[private_key.verifying_key().scalar()],
            chain_id,
        ),
        AccountType::Braavos => get_contract_address(
            salt,
            BRAAVOS_BASE_ACCOUNT_CLASS_HASH,
            &[private_key.verifying_key().scalar()],
            chain_id,
        ),
    };

    let result = if provider
        .get_class_hash_at(BlockId::Tag(Pending), address)
        .await
        .is_ok()
    {
        InvokeResponse {
            transaction_hash: Felt(FieldElement::ZERO),
        }
    } else {
        get_deployment_result(
            provider,
            account_type,
            class_hash,
            private_key,
            salt,
            chain_id,
            max_fee,
            wait_config,
        )
        .await?
    };

    update_keystore_account(account, address)?;

    Ok(result)
}

async fn deploy_from_accounts_file(
    provider: &JsonRpcClient<HttpTransport>,
    accounts_file: Utf8PathBuf,
    name: String,
    chain_id: FieldElement,
    max_fee: Option<FieldElement>,
    wait_config: WaitForTx,
) -> Result<InvokeResponse> {
    let account_data = get_account_data_from_accounts_file(&name, chain_id, &accounts_file)?;

    let private_key = SigningKey::from_secret_scalar(account_data.private_key);

    let result = get_deployment_result(
        provider,
        account_data
            .account_type
            .context("Failed to get account type from accounts file")?,
        account_data
            .class_hash
            .context("Failed to get class hash from accounts file")?,
        private_key,
        account_data
            .salt
            .context("Failed to get salt from accounts file")?,
        chain_id,
        max_fee,
        wait_config,
    )
    .await?;

    update_account_in_accounts_file(accounts_file, &name, chain_id)?;

    Ok(result)
}

#[allow(clippy::too_many_arguments)]
async fn get_deployment_result(
    provider: &JsonRpcClient<HttpTransport>,
    account_type: AccountType,
    class_hash: FieldElement,
    private_key: SigningKey,
    salt: FieldElement,
    chain_id: FieldElement,
    max_fee: Option<FieldElement>,
    wait_config: WaitForTx,
) -> Result<InvokeResponse> {
    match account_type {
        AccountType::Argent => {
            deploy_argent_account(
                provider,
                class_hash,
                private_key,
                salt,
                chain_id,
                max_fee,
                wait_config,
            )
            .await
        }
        AccountType::Oz => {
            deploy_oz_account(
                provider,
                class_hash,
                private_key,
                salt,
                chain_id,
                max_fee,
                wait_config,
            )
            .await
        }
        AccountType::Braavos => {
            deploy_braavos_account(
                provider,
                class_hash,
                private_key,
                salt,
                chain_id,
                max_fee,
                wait_config,
            )
            .await
        }
    }
}

async fn deploy_oz_account(
    provider: &JsonRpcClient<HttpTransport>,
    class_hash: FieldElement,
    private_key: SigningKey,
    salt: FieldElement,
    chain_id: FieldElement,
    max_fee: Option<FieldElement>,
    wait_config: WaitForTx,
) -> Result<InvokeResponse> {
    let factory = OpenZeppelinAccountFactory::new(
        class_hash,
        chain_id,
        LocalWallet::from_signing_key(private_key),
        provider,
    )
    .await?;

    deploy_account(factory, provider, salt, max_fee, wait_config, class_hash).await
}

async fn deploy_argent_account(
    provider: &JsonRpcClient<HttpTransport>,
    class_hash: FieldElement,
    private_key: SigningKey,
    salt: FieldElement,
    chain_id: FieldElement,
    max_fee: Option<FieldElement>,
    wait_config: WaitForTx,
) -> Result<InvokeResponse> {
    let factory = ArgentAccountFactory::new(
        class_hash,
        chain_id,
        FieldElement::ZERO,
        LocalWallet::from_signing_key(private_key),
        provider,
    )
    .await?;

    deploy_account(factory, provider, salt, max_fee, wait_config, class_hash).await
}

async fn deploy_braavos_account(
    provider: &JsonRpcClient<HttpTransport>,
    class_hash: FieldElement,
    private_key: SigningKey,
    salt: FieldElement,
    chain_id: FieldElement,
    max_fee: Option<FieldElement>,
    wait_config: WaitForTx,
) -> Result<InvokeResponse> {
    let factory = BraavosAccountFactory::new(
        class_hash,
        BRAAVOS_BASE_ACCOUNT_CLASS_HASH,
        chain_id,
        LocalWallet::from_signing_key(private_key),
        provider,
    )
    .await?;

    deploy_account(factory, provider, salt, max_fee, wait_config, class_hash).await
}

async fn deploy_account<T>(
    account_factory: T,
    provider: &JsonRpcClient<HttpTransport>,
    salt: FieldElement,
    max_fee: Option<FieldElement>,
    wait_config: WaitForTx,
    class_hash: FieldElement,
) -> Result<InvokeResponse>
where
    T: AccountFactory + Sync,
{
    let deployment = account_factory.deploy_v1(salt);

    let deploy_max_fee = if let Some(max_fee) = max_fee {
        max_fee
    } else {
        match deployment.estimate_fee().await {
            Ok(max_fee) => max_fee.overall_fee,
            Err(error) => return Err(handle_account_factory_error::<T>(error)),
        }
    };
    let result = deployment.max_fee(deploy_max_fee).send().await;

    match result {
        Err(AccountFactoryError::Provider(error)) => match error {
            StarknetError(ClassHashNotFound) => Err(anyhow!(
                "Provided class hash {:#x} does not exist",
                class_hash,
            )),
            _ => Err(handle_rpc_error(error)),
        },
        Err(_) => Err(anyhow!("Unknown AccountFactoryError")),
        Ok(result) => {
            let return_value = InvokeResponse {
                transaction_hash: Felt(result.transaction_hash),
            };
            if let Err(message) = handle_wait_for_tx(
                provider,
                result.transaction_hash,
                return_value.clone(),
                wait_config,
            )
            .await
            {
                return Err(anyhow!(message));
            }

            Ok(return_value)
        }
    }
}

fn update_account_in_accounts_file(
    accounts_file: Utf8PathBuf,
    account_name: &str,
    chain_id: FieldElement,
) -> Result<()> {
    let network_name = chain_id_to_network_name(chain_id);

    let contents =
        std::fs::read_to_string(accounts_file.clone()).context("Failed to read accounts file")?;
    let mut items: serde_json::Value = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse accounts file at = {accounts_file}"))?;
    items[&network_name][account_name]["deployed"] = serde_json::Value::from(true);
    std::fs::write(accounts_file, serde_json::to_string_pretty(&items).unwrap())
        .context("Failed to write to accounts file")?;

    Ok(())
}

fn update_keystore_account(account: &str, address: FieldElement) -> Result<()> {
    let account_path = Utf8PathBuf::from(account.to_string());
    let contents =
        std::fs::read_to_string(account_path.clone()).context("Failed to read account file")?;
    let mut items: Map<String, serde_json::Value> = serde_json::from_str(&contents)
        .map_err(|_| anyhow!("Failed to parse account file at {account_path}"))?;

    items["deployment"]["status"] = serde_json::Value::from("deployed");
    items
        .get_mut("deployment")
        .and_then(|deployment| deployment.as_object_mut())
        .expect("Failed to get deployment as an object")
        .retain(|key, _| key != "salt" && key != "context");

    items["deployment"]["address"] = format!("{address:#x}").into();

    std::fs::write(&account_path, serde_json::to_string_pretty(&items).unwrap())
        .context("Failed to write to account file")?;

    Ok(())
}
