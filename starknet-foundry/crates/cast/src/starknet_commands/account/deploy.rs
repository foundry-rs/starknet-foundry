use crate::helpers::constants::OZ_CLASS_HASH;
use anyhow::{anyhow, Result};
use camino::Utf8PathBuf;
use cast::{get_network, handle_rpc_error, parse_number, print_formatted, wait_for_tx};
use clap::Args;
use starknet::accounts::AccountFactoryError;
use starknet::accounts::{AccountFactory, OpenZeppelinAccountFactory};
use starknet::core::types::FieldElement;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use starknet::signers::{LocalWallet, SigningKey};

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
) -> Result<FieldElement> {
    let network = get_network(network)?.get_value();

    let contents = std::fs::read_to_string(path.clone())?;
    let mut items: serde_json::Value =
        serde_json::from_str(&contents).expect("failed to parse json file");
    let private_key = SigningKey::from_secret_scalar(
        parse_number(
            items[network][&name]["private_key"]
                .as_str()
                .expect("Couldn't get private key from accounts file"),
        )
        .expect("Couldn't parse private key"),
    );

    let factory = OpenZeppelinAccountFactory::new(
        parse_number(OZ_CLASS_HASH).expect("Couldn't parse OpenZeppelins account class hash"),
        provider.chain_id().await.expect("Couldn't get chain id"),
        LocalWallet::from_signing_key(private_key),
        provider,
    )
    .await?;

    let deployment = factory.deploy(
        parse_number(
            items[network][&name]["salt"]
                .as_str()
                .expect("Couldn't get salt from accounts file"),
        )
        .expect("Couldn't parse salt"),
    );
    let result = deployment.max_fee(max_fee).send().await;

    match result {
        Ok(result) => match wait_for_tx(provider, result.transaction_hash).await {
            Ok(_) => {
                items[network][&name]["deployed"] = serde_json::Value::from(false);
                std::fs::write(path, serde_json::to_string_pretty(&items).unwrap())
                    .expect("Couldn't write to accounts file");

                Ok(result.transaction_hash)
            }
            Err(message) => Err(anyhow!(message)),
        },
        Err(AccountFactoryError::Provider(error)) => handle_rpc_error(error),
        _ => Err(anyhow!("Unknown RPC error")),
    }
}

pub fn print_account_deploy_result(
    deploy_result: Result<FieldElement>,
    int_format: bool,
    json: bool,
) -> Result<()> {
    match deploy_result {
        Ok(transaction_hash) => print_formatted(
            vec![
                ("command", "Deploy account".to_string()),
                ("transaction_hash", format!("{transaction_hash}")),
            ],
            int_format,
            json,
            false,
        )?,
        Err(error) => {
            print_formatted(vec![("error", error.to_string())], int_format, json, true)?;
        }
    };
    Ok(())
}
