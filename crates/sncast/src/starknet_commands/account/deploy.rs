use anyhow::{Context, Result, anyhow, bail};
use camino::Utf8PathBuf;
use clap::Args;
use conversions::IntoConv;
use serde_json::Map;
use sncast::helpers::account::load_accounts;
use sncast::helpers::braavos::BraavosAccountFactory;
use sncast::helpers::constants::BRAAVOS_BASE_ACCOUNT_CLASS_HASH;
use sncast::helpers::fee::{FeeArgs, FeeSettings};
use sncast::helpers::rpc::RpcArgs;
use sncast::response::account::deploy::AccountDeployResponse;
use sncast::response::invoke::InvokeResponse;
use sncast::response::ui::UI;
use sncast::{
    AccountData, AccountType, SignerSource, WaitForTx, apply_optional_fields,
    chain_id_to_network_name, check_account_file_exists, get_account_data_from_accounts_file,
    get_account_data_from_keystore, handle_rpc_error, handle_wait_for_tx, ledger,
};
use starknet_rust::accounts::{AccountDeploymentV3, AccountFactory, OpenZeppelinAccountFactory};
use starknet_rust::accounts::{AccountFactoryError, ArgentAccountFactory};
use starknet_rust::core::types::BlockTag::PreConfirmed;
use starknet_rust::core::types::{BlockId, StarknetError::ClassHashNotFound};
use starknet_rust::core::utils::get_contract_address;
use starknet_rust::providers::ProviderError::StarknetError;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use starknet_rust::signers::{LocalWallet, Signer, SigningKey};
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
    accounts_file: &Utf8PathBuf,
    deploy_args: &Deploy,
    chain_id: Felt,
    wait_config: WaitForTx,
    account: &str,
    fee_args: FeeArgs,
    signer_source: &SignerSource,
    ui: &UI,
) -> Result<AccountDeployResponse> {
    match signer_source {
        SignerSource::Ledger(ledger_path) => {
            let account_name = deploy_args
                .name
                .clone()
                .ok_or_else(|| anyhow!("Required argument `--name` not provided"))?;
            deploy_from_ledger(
                provider,
                accounts_file,
                account_name,
                chain_id,
                fee_args,
                wait_config,
                ledger_path,
                ui,
            )
            .await
            .map(Into::into)
        }
        SignerSource::Keystore(keystore_path) => deploy_from_keystore(
            provider,
            chain_id,
            fee_args,
            wait_config,
            account,
            keystore_path.clone(),
            ui,
        )
        .await
        .map(Into::into),
        SignerSource::AccountsFile => {
            let account_name = deploy_args
                .name
                .clone()
                .ok_or_else(|| anyhow!("Required argument `--name` not provided"))?;
            check_account_file_exists(accounts_file)?;
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

    let private_key_felt = account_data
        .private_key
        .context("Private key not found in keystore account")?;
    let private_key = SigningKey::from_secret_scalar(private_key_felt);
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

    let result = if check_if_already_deployed(provider, address).await? {
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
    accounts_file: &Utf8PathBuf,
    name: String,
    chain_id: Felt,
    fee_args: FeeArgs,
    wait_config: WaitForTx,
    ui: &UI,
) -> Result<InvokeResponse> {
    let account_data = get_account_data_from_accounts_file(&name, chain_id, accounts_file)?;
    let (account_type, class_hash, salt) = extract_deployment_fields(&account_data)?;

    if let Some(ledger_path) = account_data.ledger_path {
        return deploy_from_ledger(
            provider,
            accounts_file,
            name,
            chain_id,
            fee_args,
            wait_config,
            &ledger_path,
            ui,
        )
        .await;
    }

    let private_key_felt = account_data
        .private_key
        .context("Private key not found. This account may require Ledger (use --ledger-path)")?;
    let private_key = SigningKey::from_secret_scalar(private_key_felt);

    let result = get_deployment_result(
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
    .await?;

    update_account_in_accounts_file(accounts_file, &name, chain_id)?;

    Ok(result)
}

#[allow(clippy::too_many_arguments)]
async fn deploy_from_ledger(
    provider: &JsonRpcClient<HttpTransport>,
    accounts_file: &Utf8PathBuf,
    name: String,
    chain_id: Felt,
    fee_args: FeeArgs,
    wait_config: WaitForTx,
    ledger_path: &str,
    ui: &UI,
) -> Result<InvokeResponse> {
    use get_account_data_from_accounts_file;

    let account_data = get_account_data_from_accounts_file(&name, chain_id, accounts_file)?;
    let (account_type, class_hash, salt) = extract_deployment_fields(&account_data)?;

    let signer = ledger::create_ledger_signer(ledger_path, ui).await?;

    let ledger_public_key = signer.get_public_key().await?.scalar();
    let stored_public_key = account_data.public_key;

    if ledger_public_key != stored_public_key {
        bail!(
            "Public key mismatch!\n\
            Ledger public key: {ledger_public_key:#x}\n\
            Stored public key: {stored_public_key:#x}\n\
            \n\
            This account was created with a different Ledger derivation path or public key.\n\
            Make sure you're using the same derivation path that was used during account creation."
        );
    }

    let stored_address = account_data
        .address
        .context("Address not found in account data")?;

    ui.print_message(
        "account deploy",
        format!("Deploying account at address {stored_address:#x}"),
    );

    if check_if_already_deployed(provider, stored_address).await? {
        return Ok(InvokeResponse {
            transaction_hash: Felt::ZERO.into_(),
        });
    }

    let result = create_factory_and_deploy(
        provider,
        account_type,
        class_hash,
        signer,
        salt,
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
    let signer = LocalWallet::from_signing_key(private_key);
    create_factory_and_deploy(
        provider,
        account_type,
        class_hash,
        signer,
        salt,
        chain_id,
        fee_args,
        wait_config,
        ui,
    )
    .await
}

#[expect(clippy::too_many_arguments)]
async fn create_factory_and_deploy<S>(
    provider: &JsonRpcClient<HttpTransport>,
    account_type: AccountType,
    class_hash: Felt,
    signer: S,
    salt: Felt,
    chain_id: Felt,
    fee_args: FeeArgs,
    wait_config: WaitForTx,
    ui: &UI,
) -> Result<InvokeResponse>
where
    S: Signer + Send + Sync,
    S::GetPublicKeyError: 'static,
    S::SignError: 'static,
{
    match account_type {
        AccountType::Argent | AccountType::Ready => {
            let factory =
                ArgentAccountFactory::new(class_hash, chain_id, None, signer, provider).await?;
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
            let factory =
                OpenZeppelinAccountFactory::new(class_hash, chain_id, signer, provider).await?;
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
                signer,
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

fn extract_deployment_fields(account_data: &AccountData) -> Result<(AccountType, Felt, Felt)> {
    let account_type = account_data
        .account_type
        .context("Failed to get account type from accounts file")?;
    let class_hash = account_data
        .class_hash
        .context("Failed to get class hash from accounts file")?;
    let salt = account_data
        .salt
        .context("Failed to get salt from accounts file")?;

    Ok((account_type, class_hash, salt))
}

async fn check_if_already_deployed(
    provider: &JsonRpcClient<HttpTransport>,
    address: Felt,
) -> Result<bool> {
    Ok(provider
        .get_class_hash_at(BlockId::Tag(PreConfirmed), address)
        .await
        .is_ok())
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
        tip,
    } = fee_settings.expect("Failed to convert to fee settings");

    let deployment = apply_optional_fields!(
        deployment,
        l1_gas => AccountDeploymentV3::l1_gas,
        l1_gas_price => AccountDeploymentV3::l1_gas_price,
        l2_gas => AccountDeploymentV3::l2_gas,
        l2_gas_price => AccountDeploymentV3::l2_gas_price,
        l1_data_gas => AccountDeploymentV3::l1_data_gas,
        l1_data_gas_price => AccountDeploymentV3::l1_data_gas_price,
        tip => AccountDeploymentV3::tip
    );
    let result = deployment.send().await;

    match result {
        Err(AccountFactoryError::Provider(error)) => match error {
            StarknetError(ClassHashNotFound) => Err(anyhow!(
                "Provided class hash {class_hash:#x} does not exist",
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
    accounts_file: &Utf8PathBuf,
    account_name: &str,
    chain_id: Felt,
) -> Result<()> {
    let network_name = chain_id_to_network_name(chain_id);

    let mut items = load_accounts(accounts_file)?;
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
