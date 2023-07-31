use crate::helpers::constants::OZ_CLASS_HASH;
use anyhow::{anyhow, bail, Result};
use camino::Utf8PathBuf;
use clap::Args;
use starknet::accounts::AccountFactoryError;
use starknet::accounts::{AccountFactory, OpenZeppelinAccountFactory};
use starknet::core::types::FieldElement;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use starknet::signers::{LocalWallet, SigningKey};

use cast::{get_network, handle_rpc_error, handle_wait_for_tx, parse_number};

use crate::helpers::response_structs::InvokeResponse;

#[derive(Args, Debug)]
#[command(about = "Deploy an account to the Starknet")]
pub struct Deploy {
    /// Name of the account to be deployed
    #[clap(short, long)]
    pub name: String,

    /// Max fee for the transaction
    #[clap(short, long)]
    pub max_fee: FieldElement,
}

pub async fn deploy(
    provider: &JsonRpcClient<HttpTransport>,
    path: Utf8PathBuf,
    name: String,
    network: &str,
    max_fee: FieldElement,
    wait: bool,
) -> Result<InvokeResponse> {
    let network_value = get_network(network)?.get_value();

    let contents = std::fs::read_to_string(path.clone()).expect("Couldn't read accounts file");
    let mut items: serde_json::Value = serde_json::from_str(&contents)
        .map_err(|_| anyhow!("Failed to parse accounts file at {path}"))?;

    if items[network_value].is_null() {
        bail!("Provided network {network} does not have any accounts defined")
    }
    if items[network_value][&name].is_null() {
        bail!("Account with name {name} does not exist")
    }

    let private_key = SigningKey::from_secret_scalar(
        parse_number(
            items
                .get(network_value)
                .and_then(|network| network.get(&name))
                .and_then(|name| name.get("private_key"))
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow!("Couldn't get private key from accounts file"))?,
        )
        .expect("Couldn't parse private key"),
    );

    let factory = OpenZeppelinAccountFactory::new(
        parse_number(OZ_CLASS_HASH).expect("Couldn't parse OpenZeppelin's account class hash"),
        provider.chain_id().await.expect("Couldn't get chain id"),
        LocalWallet::from_signing_key(private_key),
        provider,
    )
    .await?;

    let deployment = factory.deploy(
        parse_number(
            items
                .get(network_value)
                .and_then(|network| network.get(&name))
                .and_then(|name| name.get("salt"))
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow!("Couldn't get salt from accounts file"))?,
        )
        .expect("Couldn't parse salt"),
    );
    let result = deployment.max_fee(max_fee).send().await;

    match result {
        Err(AccountFactoryError::Provider(error)) => handle_rpc_error(error),
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

            items[network_value][&name]["deployed"] = serde_json::Value::from(true);
            std::fs::write(path, serde_json::to_string_pretty(&items).unwrap())
                .expect("Couldn't write to accounts file");

            Ok(return_value)
        }
    }
}
