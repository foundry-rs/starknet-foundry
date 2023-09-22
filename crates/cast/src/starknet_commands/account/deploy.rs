use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8PathBuf;
use cast::helpers::constants::{KEYSTORE_PASSWORD_ENV_VAR, OZ_CLASS_HASH};
use clap::Args;
use serde_json::Map;
use starknet::accounts::AccountFactoryError;
use starknet::accounts::{AccountFactory, OpenZeppelinAccountFactory};
use starknet::core::types::BlockTag::Pending;
use starknet::core::types::{BlockId, FieldElement, StarknetError};
use starknet::core::utils::get_contract_address;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::ProviderError::{self};
use starknet::providers::{
    JsonRpcClient, MaybeUnknownErrorCode, Provider, StarknetErrorWithMessage,
};
use starknet::signers::{LocalWallet, SigningKey};

use cast::{
    account_file_exists, chain_id_to_network_name, get_keystore_password, handle_rpc_error,
    handle_wait_for_tx, parse_number,
};

use cast::helpers::response_structs::InvokeResponse;

#[derive(Args, Debug)]
#[command(about = "Deploy an account to the Starknet")]
pub struct Deploy {
    /// Name of the account to be deployed
    #[clap(short, long)]
    pub name: Option<String>,

    /// Max fee for the transaction
    #[clap(short, long)]
    pub max_fee: FieldElement,

    /// Custom open zeppelin contract class hash of declared contract
    #[clap(short, long)]
    pub class_hash: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub async fn deploy(
    provider: &JsonRpcClient<HttpTransport>,
    accounts_file: Utf8PathBuf,
    name: String,
    chain_id: FieldElement,
    max_fee: FieldElement,
    wait: bool,
    class_hash: Option<String>,
    keystore_path: Option<Utf8PathBuf>,
    account_path: Option<Utf8PathBuf>,
) -> Result<InvokeResponse> {
    if let Some(keystore_path_) = keystore_path {
        let account_path_ = account_path.ok_or_else(|| {
            anyhow!("--account must be passed and be a path when using --keystore")
        })?;

        deploy_from_keystore(
            provider,
            chain_id,
            max_fee,
            wait,
            keystore_path_,
            account_path_,
        )
        .await
    } else {
        if name == String::default() {
            bail!("No --name value passed")
        }
        account_file_exists(&accounts_file)?;
        deploy_from_accounts_file(
            provider,
            accounts_file,
            name,
            chain_id,
            max_fee,
            wait,
            class_hash,
        )
        .await
    }
}

async fn deploy_from_keystore(
    provider: &JsonRpcClient<HttpTransport>,
    chain_id: FieldElement,
    max_fee: FieldElement,
    wait: bool,
    keystore_path: Utf8PathBuf,
    account_path: Utf8PathBuf,
) -> Result<InvokeResponse> {
    let contents =
        std::fs::read_to_string(account_path.clone()).context("Couldn't read account file")?;
    let mut items: Map<String, serde_json::Value> = serde_json::from_str(&contents)
        .map_err(|_| anyhow!("Failed to parse account file at {account_path}"))?;

    let deployment = items
        .get("deployment")
        .ok_or_else(|| anyhow!("No deployment field in account JSON file"))?;

    let status = deployment
        .get("status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("Failed to get status from account JSON file"))?;

    if status == "deployed" {
        bail!("Account already deployed");
    }

    let salt = FieldElement::from_hex_be(
        deployment
            .get("salt")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Failed to get salt from account JSON file"))?,
    )?;
    let oz_class_hash = FieldElement::from_hex_be(
        deployment
            .get("class_hash")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Failed to get class_hash from account JSON file"))?,
    )?;

    if !keystore_path.exists() {
        bail!("Couldn't read keystore file");
    }
    let private_key = SigningKey::from_keystore(
        keystore_path,
        get_keystore_password(KEYSTORE_PASSWORD_ENV_VAR)?.as_str(),
    )?;
    let public_key: FieldElement = {
        let pk = items
            .get("variant")
            .and_then(|v| v.get("public_key"))
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("No public_key in account JSON file"))?;
        parse_number(pk)?
    };
    if public_key != private_key.verifying_key().scalar() {
        bail!("Public key and private key from keystore do not match");
    }

    let address = get_contract_address(
        salt,
        oz_class_hash,
        &[private_key.verifying_key().scalar()],
        FieldElement::ZERO,
    );

    let result = if provider
        .get_class_hash_at(BlockId::Tag(Pending), address)
        .await
        .is_ok()
    {
        println!("Account already deployed, adjusting the account file");
        InvokeResponse {
            transaction_hash: FieldElement::ZERO,
        }
    } else {
        deploy_oz_account(
            provider,
            oz_class_hash,
            private_key,
            salt,
            chain_id,
            max_fee,
            wait,
        )
        .await?
    };

    items["deployment"]["status"] = serde_json::Value::from("deployed");
    items.get_mut("deployment").and_then(|deployment| {
        deployment
            .as_object_mut()
            .expect("should be an object")
            .remove("salt")
    });
    items["deployment"]["address"] = format!("{address:#x}").into();

    std::fs::write(&account_path, serde_json::to_string_pretty(&items).unwrap())
        .context("Couldn't write to account file")?;

    Ok(result)
}

async fn deploy_from_accounts_file(
    provider: &JsonRpcClient<HttpTransport>,
    accounts_file: Utf8PathBuf,
    name: String,
    chain_id: FieldElement,
    max_fee: FieldElement,
    wait: bool,
    class_hash: Option<String>,
) -> Result<InvokeResponse> {
    let network_name = chain_id_to_network_name(chain_id);

    let contents =
        std::fs::read_to_string(accounts_file.clone()).context("Couldn't read accounts file")?;
    let mut items: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|_| anyhow!("Failed to parse accounts file at {accounts_file}"))?;

    if items[&network_name].is_null() {
        bail!("No accounts defined for network {}", network_name);
    }
    if items[&network_name][&name].is_null() {
        bail!("Account with name {name} does not exist")
    }
    let account = &items[&network_name][&name];

    let private_key = SigningKey::from_secret_scalar(
        parse_number(
            account
                .get("private_key")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow!("Couldn't get private key from accounts file"))?,
        )
        .context("Couldn't parse private key")?,
    );

    let oz_class_hash = {
        if let Some(class_hash_) = &class_hash {
            class_hash_.as_str()
        } else if let Some(class_hash_) = account
            .get("class_hash")
            .and_then(serde_json::Value::as_str)
        {
            class_hash_
        } else {
            OZ_CLASS_HASH
        }
    };

    let result = deploy_oz_account(
        provider,
        parse_number(oz_class_hash).context("Couldn't parse account class hash")?,
        private_key,
        parse_number(
            account
                .get("salt")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow!("Couldn't get salt from accounts file"))?,
        )
        .context("Couldn't parse salt")?,
        chain_id,
        max_fee,
        wait,
    )
    .await?;

    items[&network_name][&name]["deployed"] = serde_json::Value::from(true);
    std::fs::write(accounts_file, serde_json::to_string_pretty(&items).unwrap())
        .context("Couldn't write to accounts file")?;

    Ok(result)
}

async fn deploy_oz_account(
    provider: &JsonRpcClient<HttpTransport>,
    oz_class_hash: FieldElement,
    private_key: SigningKey,
    salt: FieldElement,
    chain_id: FieldElement,
    max_fee: FieldElement,
    wait: bool,
) -> Result<InvokeResponse> {
    let factory = OpenZeppelinAccountFactory::new(
        oz_class_hash,
        chain_id,
        LocalWallet::from_signing_key(private_key),
        provider,
    )
    .await?;

    let deployment = factory.deploy(salt);
    let result = deployment.max_fee(max_fee).send().await;

    match result {
        Err(AccountFactoryError::Provider(error)) => match error {
            ProviderError::StarknetError(StarknetErrorWithMessage {
                code: MaybeUnknownErrorCode::Known(StarknetError::ClassHashNotFound),
                message: _,
            }) => Err(anyhow!(
                "Provided class hash {:#x} does not exist",
                oz_class_hash,
            )),
            _ => handle_rpc_error(error),
        },
        Err(_) => Err(anyhow!("Unknown RPC error")),
        Ok(result) => {
            let return_value = InvokeResponse {
                transaction_hash: result.transaction_hash,
            };
            if let Err(message) = handle_wait_for_tx(
                provider,
                result.transaction_hash,
                return_value.clone(),
                wait,
            )
            .await
            {
                return Err(anyhow!(message));
            }

            Ok(return_value)
        }
    }
}
