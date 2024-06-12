use crate::helpers::constants::BRAAVOS_BASE_ACCOUNT_CLASS_HASH;
use crate::response::structs::{Felt, InvokeResponse};
use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8PathBuf;
use clap::Args;
use starknet::accounts::AccountFactoryError;
use starknet::core::types::BlockTag::Pending;
use starknet::core::types::{BlockId, FieldElement, StarknetError::ClassHashNotFound};
use starknet::core::utils::get_contract_address;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::ProviderError::StarknetError;
use starknet::providers::{JsonRpcClient, Provider};
use starknet::signers::{LocalWallet, SigningKey};

use crate::starknet_commands::account::account_factory::{create_account_factory, AccountFactory};
use crate::starknet_commands::account::accounts_format::{
    AccountData, AccountKeystore, AccountType,
};
use crate::{
    chain_id_to_network_name, get_account_data_from_accounts_file, get_account_data_from_keystore,
    handle_account_factory_error, handle_rpc_error, handle_wait_for_tx, WaitForTx,
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
    if let Some(keystore_path) = keystore_path {
        let account_data = get_account_data_from_keystore(account, &keystore_path)?;

        if let Some(true) = account_data.deployed {
            bail!("Account already deployed");
        }

        let account_data = AccountData {
            address: Some(get_account_address(&account_data, chain_id)?),
            ..account_data
        };
        let (result, account_data) = deploy_from_data(
            provider,
            account_data,
            chain_id,
            deploy_args.max_fee,
            wait_config,
        )
        .await?;

        update_keystore_account(account, &AccountKeystore::try_from(&account_data)?)?;

        Ok(result)
    } else {
        let account_name = deploy_args
            .name
            .ok_or_else(|| anyhow!("Required argument `--name` not provided"))?;
        let account_data =
            get_account_data_from_accounts_file(&account_name, chain_id, &accounts_file)?;

        let (result, account_data) = deploy_from_data(
            provider,
            account_data,
            chain_id,
            deploy_args.max_fee,
            wait_config,
        )
        .await?;
        update_account_in_accounts_file(accounts_file, &account_name, chain_id, account_data)?;

        Ok(result)
    }
}

async fn deploy_from_data(
    provider: &JsonRpcClient<HttpTransport>,
    account_data: AccountData,
    chain_id: FieldElement,
    max_fee: Option<FieldElement>,
    wait_config: WaitForTx,
) -> Result<(InvokeResponse, AccountData)> {
    if let Some(true) = account_data.deployed {
        bail!("Account already deployed");
    }

    let private_key = SigningKey::from_secret_scalar(account_data.private_key);
    if account_data.public_key != private_key.verifying_key().scalar() {
        bail!("Public key and private key from keystore do not match");
    }

    let result = if provider
        .get_class_hash_at(
            BlockId::Tag(Pending),
            account_data.address.context("Failed to get address")?,
        )
        .await
        .is_ok()
    {
        InvokeResponse {
            transaction_hash: Felt(FieldElement::ZERO),
        }
    } else {
        let account_type = account_data
            .account_type
            .clone()
            .context("Failed to get account type")?;
        let class_hash = account_data
            .class_hash
            .context("Failed to get class hash")?;
        let salt = account_data.salt.context("Failed to get salt")?;

        let factory = create_account_factory(
            account_type,
            class_hash,
            chain_id,
            LocalWallet::from_signing_key(private_key),
            provider,
        )
        .await
        .context("Failed to create account factory")?;

        match factory {
            AccountFactory::Oz(factory) => {
                deploy_account(factory, provider, salt, max_fee, wait_config, class_hash).await
            }
            AccountFactory::Argent(factory) => {
                deploy_account(factory, provider, salt, max_fee, wait_config, class_hash).await
            }
            AccountFactory::Braavos(factory) => {
                deploy_account(factory, provider, salt, max_fee, wait_config, class_hash).await
            }
        }?
    };

    let account_data = AccountData {
        deployed: Some(true),
        ..account_data
    };
    Ok((result, account_data))
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
    T: starknet::accounts::AccountFactory + Sync,
{
    let deployment = account_factory.deploy(salt);

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

fn get_account_address(account_data: &AccountData, chain_id: FieldElement) -> Result<FieldElement> {
    let salt = account_data
        .salt
        .context("Failed to get salt from keystore format")?;
    let account_type = account_data
        .account_type
        .clone()
        .context("Failed to get account type from keystore format")?;
    let class_hash = account_data
        .class_hash
        .context("Failed to get class hash from keystore format")?;

    let address = match account_type {
        AccountType::Argent => get_contract_address(
            salt,
            class_hash,
            &[account_data.public_key, FieldElement::ZERO],
            FieldElement::ZERO,
        ),
        AccountType::Oz => {
            get_contract_address(salt, class_hash, &[account_data.public_key], chain_id)
        }
        AccountType::Braavos => get_contract_address(
            salt,
            BRAAVOS_BASE_ACCOUNT_CLASS_HASH,
            &[account_data.private_key],
            chain_id,
        ),
    };
    Ok(address)
}

fn update_account_in_accounts_file(
    accounts_file: Utf8PathBuf,
    account_name: &str,
    chain_id: FieldElement,
    account_data: AccountData,
) -> Result<()> {
    let network_name = chain_id_to_network_name(chain_id);

    let contents =
        std::fs::read_to_string(accounts_file.clone()).context("Failed to read accounts file")?;
    let mut items: serde_json::Value = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse accounts file at = {accounts_file}"))?;
    items[&network_name][account_name] = serde_json::to_value(account_data)?;
    std::fs::write(accounts_file, serde_json::to_string_pretty(&items).unwrap())
        .context("Failed to write to accounts file")?;

    Ok(())
}

fn update_keystore_account(account: &str, account_data: &AccountKeystore) -> Result<()> {
    let account_path = Utf8PathBuf::from(account.to_string());
    std::fs::write(
        account_path,
        serde_json::to_string_pretty(&account_data).unwrap(),
    )
    .context("Failed to write to account file")?;

    Ok(())
}
