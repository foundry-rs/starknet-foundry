use std::str::FromStr;

use crate::starknet_commands::invoke::execute_calls;
use crate::starknet_commands::multicall::contract_registry::ContractRegistry;
use crate::starknet_commands::multicall::deploy::MulticallDeploy;
use crate::starknet_commands::multicall::invoke::MulticallInvoke;
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use clap::Args;
use serde::Deserialize;
use serde_json::Number;
use sncast::WaitForTx;
use sncast::helpers::fee::FeeArgs;
use sncast::helpers::rpc::RpcArgs;
use sncast::response::errors::handle_starknet_command_error;
use sncast::response::multicall::run::MulticallRunResponse;
use sncast::response::ui::UI;
use starknet_rust::accounts::SingleOwnerAccount;
use starknet_rust::core::types::Call;
use starknet_rust::providers::JsonRpcClient;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::signers::LocalWallet;
use starknet_types_core::felt::Felt;

#[derive(Args, Debug, Clone)]
#[command(about = "Execute a multicall from a .toml file", long_about = None)]
pub struct Run {
    /// Path to the toml file with declared operations
    #[arg(short = 'p', long = "path")]
    pub path: Utf8PathBuf,

    #[command(flatten)]
    pub fee_args: FeeArgs,

    #[command(flatten)]
    pub rpc: RpcArgs,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Input {
    String(String),
    Number(Number),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "call_type", rename_all = "lowercase")]
enum CallItem {
    Deploy(DeployItem),
    Invoke(InvokeItem),
}

#[derive(Deserialize, Debug)]
struct MulticallFile {
    #[serde(rename = "call")]
    calls: Vec<CallItem>,
}

#[derive(Deserialize, Debug)]
pub struct DeployItem {
    pub class_hash: Felt,
    pub inputs: Vec<Input>,
    pub unique: bool,
    pub salt: Option<Felt>,
    pub id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct InvokeItem {
    pub contract_address: String,
    pub function: String,
    pub inputs: Vec<Input>,
}

pub async fn run(
    run: Box<Run>,
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    provider: &JsonRpcClient<HttpTransport>,
    wait_config: WaitForTx,
    ui: &UI,
) -> Result<MulticallRunResponse> {
    let fee_args = run.fee_args.clone();

    let contents = std::fs::read_to_string(&run.path)?;
    let multicall: MulticallFile =
        toml::from_str(&contents).with_context(|| format!("Failed to parse {}", run.path))?;

    let mut contracts = ContractRegistry::new(provider);
    let mut parsed_calls: Vec<Call> = vec![];

    for call in multicall.calls {
        match call {
            CallItem::Deploy(item) => {
                let deploy = MulticallDeploy::new_from_item(&item, &contracts)?;
                let call = deploy.build_call(account, &mut contracts).await?;
                parsed_calls.push(call);
            }
            CallItem::Invoke(item) => {
                let invoke = MulticallInvoke::new_from_item(&item, &contracts)?;
                let call = invoke.build_call(&mut contracts).await?;
                parsed_calls.push(call);
            }
        }
    }

    execute_calls(account, parsed_calls, fee_args, None, wait_config, ui)
        .await
        .map(Into::into)
        .map_err(handle_starknet_command_error)
}

pub fn parse_inputs(inputs: &[Input], contract_registry: &ContractRegistry) -> Result<Vec<Felt>> {
    let mut parsed_inputs = Vec::new();
    for input in inputs {
        let felt_value = match input {
            Input::String(s) => contract_registry
                .get_address_by_id(s)
                .map_or_else(|| s.parse(), Ok)?,
            Input::Number(n) => Felt::from_str(&n.to_string())
                .with_context(|| format!("Failed to parse {} to felt", n))?,
        };
        parsed_inputs.push(felt_value);
    }

    Ok(parsed_inputs)
}
