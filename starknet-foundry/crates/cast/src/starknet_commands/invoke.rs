use anyhow::{anyhow, Context, Result};
use clap::Args;

use cast::print_formatted;
use cast::{handle_rpc_error, handle_wait_for_tx_result};
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{Account, Call, ConnectedAccount, SingleOwnerAccount};
use starknet::core::types::FieldElement;
use starknet::core::utils::get_selector_from_name;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use starknet::signers::LocalWallet;

use cast::parse_number;

#[derive(Args)]
#[command(about = "Invoke a contract on Starknet")]
pub struct Invoke {
    /// Address of contract to invoke
    #[clap(short = 'a', long)]
    pub contract_address: String,

    /// Name of the function to invoke
    #[clap(short, long)]
    pub entry_point_name: String,

    /// Calldata for the invoked function
    #[clap(short, long, value_delimiter = ' ')]
    pub calldata: Vec<String>,

    /// Max fee for the transaction. If not provided, max fee will be automatically estimated
    #[clap(short, long)]
    pub max_fee: Option<u128>,
}

pub async fn invoke_and_print(
    contract_address: &str,
    entry_point_name: &str,
    calldata: Vec<&str>,
    max_fee: Option<u128>,
    account: &mut SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    int_format: bool,
    json: bool,
) -> Result<()> {
    let result = invoke(
        contract_address,
        entry_point_name,
        calldata,
        max_fee,
        account,
    )
    .await;
    match result {
        Ok(transaction_hash) => print_formatted(
            vec![
                ("command", "Invoke".to_string()),
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

pub async fn invoke(
    contract_address: &str,
    entry_point_name: &str,
    calldata: Vec<&str>,
    max_fee: Option<u128>,
    account: &mut SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
) -> Result<FieldElement> {
    let call = Call {
        to: parse_number(contract_address)?,
        selector: get_selector_from_name(entry_point_name)?,
        calldata: calldata
            .iter()
            .map(|cd| parse_number(cd).context("Failed to convert calldata to FieldElement"))
            .collect::<Result<Vec<_>>>()?,
    };
    let execution = account.execute(vec![call]);

    let execution = if let Some(max_fee) = max_fee {
        execution.max_fee(FieldElement::from(max_fee))
    } else {
        execution
    };

    let result = execution.send().await;

    match result {
        Ok(result) => {
            handle_wait_for_tx_result(
                account.provider(),
                result.transaction_hash,
                result.transaction_hash,
            )
            .await
        }
        Err(Provider(error)) => handle_rpc_error(error),
        _ => Err(anyhow!("Unknown RPC error")),
    }
}
