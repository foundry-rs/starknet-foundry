use anyhow::{Context, Result, anyhow, bail};
use camino::Utf8PathBuf;
use clap::Args;
use conversions::IntoConv;
use foundry_ui::UI;
use serde_json::Map;
use sncast::helpers::account::load_accounts;
use sncast::helpers::braavos::BraavosAccountFactory;
use sncast::helpers::constants::{BRAAVOS_BASE_ACCOUNT_CLASS_HASH, KEYSTORE_PASSWORD_ENV_VAR};
use sncast::helpers::fee::{FeeArgs, FeeSettings};
use sncast::helpers::rpc::RpcArgs;
use sncast::response::account::deploy::AccountDeployResponse;
use sncast::response::invoke::InvokeResponse;
use sncast::{
    AccountType, WaitForTx, apply_optional_fields, chain_id_to_network_name,
    check_account_file_exists, get_account_data_from_accounts_file, get_account_data_from_keystore,
    get_keystore_password, handle_rpc_error, handle_wait_for_tx,
};
use starknet::accounts::{AccountDeploymentV3, AccountFactory, OpenZeppelinAccountFactory};
use starknet::accounts::{AccountFactoryError, ArgentAccountFactory};
use starknet::core::types::BlockTag::Pending;
use starknet::core::types::{BlockId, StarknetError::ClassHashNotFound};
use starknet::core::utils::get_contract_address;
use starknet::providers::ProviderError::StarknetError;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use starknet::signers::{LocalWallet, SigningKey};
use starknet_types_core::felt::Felt;

#[derive(Args, Debug)]
#[command(about = "Deploy an account to the Starknet")]
pub struct Deploy {
    /// Name of the account to be deployed
    #[arg(short, long)]
    pub name: Option<String>,

    #[command(flatten)]
    pub fee_args: FeeArgs,

    #[command(flatten)]
    pub rpc: RpcArgs,

    /// If passed, the command will not trigger an interactive prompt to add an account as a default
    #[arg(long)]
    pub silent: bool,
}

#[expect(clippy::too_many_arguments)]
pub async fn deploy(
    provider: &JsonRpcClient<HttpTransport>,
    accounts_file: Utf8PathBuf,
    deploy_args: &Deploy,
    chain_id: Felt,
    wait_config: WaitForTx,
    account: &str,
    keystore_path: Option<Utf8PathBuf>,
    fee_args: FeeArgs,
    ui: &UI,
) -> Result<AccountDeployResponse> {
    if let Some(keystore_path_) = keystore_path {
        deploy_from_keystore(
            provider,
            chain_id,
            fee_args,
            wait_config,
            account,
            keystore_path_,
            ui,
        )
        .await
        .map(Into::into)
    } else {
        let account_name = deploy_args
            .name
            .clone()
            .ok_or_else(|| anyhow!("Required argument `--name` not provided"))?;
        check_account_file_exists(&accounts_file)?;
        deploy_from_accounts_file(
            provider,
            accounts_file,
            account_name,
            chain_id,
            fee_args,
            wait_config,
            ui,
        )
        .await
        .map(Into::into)
    }
}

async fn deploy_from_keystore(
    provider: &JsonRpcClient<HttpTransport>,
    chain_id: Felt,
    fee_args: FeeArgs,
    wait_config: WaitForTx,
    account: &str,
    keystore_path: Utf8PathBuf,
    ui: &UI,
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

    let address = compute_account_address(salt, &private_key, class_hash, account_type, chain_id);

    let result = if provider
        .get_class_hash_at(BlockId::Tag(Pending), address)
        .await
        .is_ok()
    {
        InvokeResponse {
            transaction_hash: Felt::ZERO.into_(),
        }
    } else {
        get_deployment_result(
            provider,
            account_type,
            class_hash,
            private_key,
            salt,
            chain_id,
            fee_args,
            wait_config,
            ui,
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
    chain_id: Felt,
    fee_args: FeeArgs,
    wait_config: WaitForTx,
    ui: &UI,
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
        fee_args,
        wait_config,
        ui,
    )
    .await?;

    update_account_in_accounts_file(accounts_file, &name, chain_id)?;

    Ok(result)
}

#[expect(clippy::too_many_arguments)]
async fn get_deployment_result(
    provider: &JsonRpcClient<HttpTransport>,
    account_type: AccountType,
    class_hash: Felt,
    private_key: SigningKey,
    salt: Felt,
    chain_id: Felt,
    fee_args: FeeArgs,
    wait_config: WaitForTx,
    ui: &UI,
) -> Result<InvokeResponse> {
    match account_type {
        AccountType::Argent | AccountType::Ready => {
            let factory = ArgentAccountFactory::new(
                class_hash,
                chain_id,
                None,
                LocalWallet::from_signing_key(private_key),
                provider,
            )
            .await?;

            deploy_account(
                factory,
                provider,
                salt,
                fee_args,
                wait_config,
                class_hash,
                ui,
            )
            .await
        }
        AccountType::OpenZeppelin => {
            let factory = OpenZeppelinAccountFactory::new(
                class_hash,
                chain_id,
                LocalWallet::from_signing_key(private_key),
                provider,
            )
            .await?;

            deploy_account(
                factory,
                provider,
                salt,
                fee_args,
                wait_config,
                class_hash,
                ui,
            )
            .await
        }
        AccountType::Braavos => {
            let factory = BraavosAccountFactory::new(
                class_hash,
                BRAAVOS_BASE_ACCOUNT_CLASS_HASH,
                chain_id,
                LocalWallet::from_signing_key(private_key),
                provider,
            )
            .await?;

            deploy_account(
                factory,
                provider,
                salt,
                fee_args,
                wait_config,
                class_hash,
                ui,
            )
            .await
        }
    }
}

async fn deploy_account<T>(
    account_factory: T,
    provider: &JsonRpcClient<HttpTransport>,
    salt: Felt,
    fee_args: FeeArgs,
    wait_config: WaitForTx,
    class_hash: Felt,
    ui: &UI,
) -> Result<InvokeResponse>
where
    T: AccountFactory + Sync,
{
    let deployment = account_factory.deploy_v3(salt);

    let fee_settings = if fee_args.max_fee.is_some() {
        let fee_estimate = deployment
            .estimate_fee()
            .await
            .expect("Failed to estimate fee");
        fee_args.try_into_fee_settings(Some(&fee_estimate))
    } else {
        fee_args.try_into_fee_settings(None)
    };

    let FeeSettings {
        l1_gas,
        l1_gas_price,
        l2_gas,
        l2_gas_price,
        l1_data_gas,
        l1_data_gas_price,
    } = fee_settings.expect("Failed to convert to fee settings");

    let deployment = apply_optional_fields!(
        deployment,
        l1_gas => AccountDeploymentV3::l1_gas,
        l1_gas_price => AccountDeploymentV3::l1_gas_price,
        l2_gas => AccountDeploymentV3::l2_gas,
        l2_gas_price => AccountDeploymentV3::l2_gas_price,
        l1_data_gas => AccountDeploymentV3::l1_data_gas,
        l1_data_gas_price => AccountDeploymentV3::l1_data_gas_price
    );
    let result = deployment.send().await;

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
                transaction_hash: result.transaction_hash.into_(),
            };
            if let Err(message) = handle_wait_for_tx(
                provider,
                result.transaction_hash,
                return_value.clone(),
                wait_config,
                ui,
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
    chain_id: Felt,
) -> Result<()> {
    let network_name = chain_id_to_network_name(chain_id);

    let mut items = load_accounts(&accounts_file)?;
    items[&network_name][account_name]["deployed"] = serde_json::Value::from(true);
    std::fs::write(accounts_file, serde_json::to_string_pretty(&items).unwrap())
        .context("Failed to write to accounts file")?;

    Ok(())
}

fn update_keystore_account(account: &str, address: Felt) -> Result<()> {
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

pub(crate) fn compute_account_address(
    salt: Felt,
    private_key: &SigningKey,
    class_hash: Felt,
    account_type: AccountType,
    chain_id: Felt,
) -> Felt {
    match account_type {
        AccountType::Argent | AccountType::Ready => get_contract_address(
            salt,
            class_hash,
            &[private_key.verifying_key().scalar(), Felt::ZERO],
            Felt::ZERO,
        ),
        AccountType::OpenZeppelin => get_contract_address(
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
    }
}
