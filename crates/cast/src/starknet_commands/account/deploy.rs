use anyhow::{anyhow, bail, Context, Result};
use camino::Utf8PathBuf;
use cast::helpers::constants::OZ_CLASS_HASH;
use clap::Args;
use starknet::accounts::AccountFactoryError;
use starknet::accounts::{AccountFactory, OpenZeppelinAccountFactory};
use starknet::core::types::{FieldElement, StarknetError};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::ProviderError::{self};
use starknet::providers::{JsonRpcClient, MaybeUnknownErrorCode, StarknetErrorWithMessage};
use starknet::signers::{LocalWallet, SigningKey};

use cast::{chain_id_to_network_name, handle_rpc_error, handle_wait_for_tx, parse_number};

use cast::helpers::response_structs::InvokeResponse;

#[derive(Args, Debug)]
#[command(about = "Deploy an account to the Starknet")]
pub struct Deploy {
    /// Name of the account to be deployed
    #[clap(short, long)]
    pub name: String,

    /// Max fee for the transaction
    #[clap(short, long)]
    pub max_fee: FieldElement,

    /// Custom open zeppelin contract class hash of declared contract
    #[clap(short, long)]
    pub class_hash: Option<String>,
}

pub async fn deploy(
    provider: &JsonRpcClient<HttpTransport>,
    path: Utf8PathBuf,
    name: String,
    chain_id: FieldElement,
    max_fee: FieldElement,
    wait: bool,
    class_hash: Option<String>,
) -> Result<InvokeResponse> {
    let network_name = chain_id_to_network_name(chain_id);

    let contents = std::fs::read_to_string(path.clone()).context("Couldn't read accounts file")?;
    let mut items: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|_| anyhow!("Failed to parse accounts file at {path}"))?;

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

    let factory = OpenZeppelinAccountFactory::new(
        parse_number(oz_class_hash).context("Couldn't parse account class hash")?,
        chain_id,
        LocalWallet::from_signing_key(private_key),
        provider,
    )
    .await?;

    let deployment = factory.deploy(
        parse_number(
            account
                .get("salt")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow!("Couldn't get salt from accounts file"))?,
        )
        .context("Couldn't parse salt")?,
    );
    let result = deployment.max_fee(max_fee).send().await;

    match result {
        Err(AccountFactoryError::Provider(error)) => match error {
            ProviderError::StarknetError(StarknetErrorWithMessage {
                code: MaybeUnknownErrorCode::Known(StarknetError::ClassHashNotFound),
                message: _,
            }) => Err(anyhow!(
                "Provided class hash {} does not exist",
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

            items[&network_name][&name]["deployed"] = serde_json::Value::from(true);
            std::fs::write(path, serde_json::to_string_pretty(&items).unwrap())
                .context("Couldn't write to accounts file")?;

            Ok(return_value)
        }
    }
}
