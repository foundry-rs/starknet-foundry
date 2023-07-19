use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use cast::{handle_rpc_error, handle_wait_for_tx_result};
use clap::Args;
use scarb::ops::find_manifest_path;
use serde::Deserialize;
use starknet::accounts::AccountError::Provider;
use starknet::accounts::ConnectedAccount;
use starknet::core::types::FieldElement;
use starknet::{
    accounts::{Account, SingleOwnerAccount},
    core::types::{
        contract::{CompiledClass, SierraClass},
        DeclareTransactionResult,
    },
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::LocalWallet,
};
use std::process::{Command, Stdio};
use std::sync::Arc;

#[allow(dead_code)]
#[derive(Deserialize)]
struct ScarbStarknetArtifacts {
    version: u32,
    contracts: Vec<ScarbStarknetContract>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ScarbStarknetContract {
    id: String,
    package_name: String,
    contract_name: String,
    artifacts: ScarbStarknetContractArtifact,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ScarbStarknetContractArtifact {
    sierra: Utf8PathBuf,
    casm: Option<Utf8PathBuf>,
}

#[derive(Args)]
#[command(about = "Declare a contract to starknet", long_about = None)]
pub struct Declare {
    /// contract name
    #[clap(short = 'c', long = "contract-name")]
    pub contract: String,

    /// Max fee for the transaction. If not provided, max fee will be automatically estimated
    #[clap(short, long)]
    pub max_fee: Option<FieldElement>,
}

#[allow(clippy::too_many_lines)]
pub async fn declare(
    contract_name: &str,
    max_fee: Option<FieldElement>,
    account: &mut SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
) -> Result<DeclareTransactionResult> {
    let contract_name: String = contract_name.to_string();
    which::which("scarb")
        .context("Cannot find `scarb` binary in PATH. Make sure you have Scarb installed https://github.com/software-mansion/scarb")?;
    let command_result = Command::new("scarb")
        .current_dir(std::env::current_dir().context("Failed to obtain current dir")?)
        .arg("--release")
        .arg("build")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to start building contracts with Scarb")?;
    let result_code = command_result
        .status
        .code()
        .context("failed to obtain status code from scarb build")?;
    if result_code != 0 {
        anyhow::bail!("scarb build returned non-zero exit code: {}", result_code);
    }

    // TODO #41 improve handling starknet artifacts
    let compiled_directory = find_manifest_path(None)
        .expect("Failed to obtain Scarb.toml file path")
        .parent()
        .map(|parent| parent.join("target/release"))
        .ok_or_else(|| anyhow!("Failed to obtain the path to compiled contracts"))?;

    let mut paths = match compiled_directory.read_dir() {
        Ok(paths) => paths,
        Err(err) => {
            return Err(anyhow!(
                "Failed to read ./target/release, scarb build probably failed: {}",
                err
            ))
        }
    };

    let starknet_artifacts = paths
        .find_map(|path| {
            path.ok().and_then(|entry| {
                let name = entry.file_name().into_string().ok()?;
                name.contains("starknet_artifacts").then_some(entry.path())
            })
        })
        .ok_or(anyhow!("Failed to find starknet_artifacts.json file"))?;

    let starknet_artifacts = match std::fs::read_to_string(starknet_artifacts) {
        Ok(content) => content,
        Err(err) => {
            return Err(anyhow!(
                "Failed to read starknet_artifacts.json contents: {}",
                err
            ))
        }
    };

    let starknet_artifacts: ScarbStarknetArtifacts =
        serde_json::from_str(starknet_artifacts.as_str())
            .context("Failed to parse starknet_artifacts.json contents")?;

    let sierra_path = starknet_artifacts
        .contracts
        .iter()
        .find_map(|contract| {
            if contract.contract_name == contract_name {
                return Some(contract.artifacts.sierra.clone());
            }
            None
        })
        .unwrap_or_else(|| {
            panic!("Failed to find contract {contract_name} in starknet_artifacts.json")
        });
    let sierra_contract_path = &compiled_directory.join(sierra_path);

    let casm_path = starknet_artifacts
        .contracts
        .iter()
        .find_map(|contract| {
            if contract.contract_name == contract_name {
                return Some(contract.artifacts.casm.clone());
            }
            None
        })
        .unwrap_or_else(|| {
            panic!("Failed to find contract {contract_name} in starknet_artifacts.json")
        })
        .unwrap();
    let casm_contract_path = &compiled_directory.join(casm_path);

    let contract_definition: SierraClass = {
        let file_contents = std::fs::read(sierra_contract_path.clone())
            .with_context(|| format!("Failed to read contract file: {sierra_contract_path}"))?;
        serde_json::from_slice(&file_contents).with_context(|| {
            format!("Failed to parse contract definition: {sierra_contract_path}")
        })?
    };
    let casm_contract_definition: CompiledClass = {
        let file_contents = std::fs::read(casm_contract_path.clone())
            .with_context(|| format!("Failed to read contract file: {casm_contract_path}"))?;
        serde_json::from_slice(&file_contents)
            .with_context(|| format!("Failed to parse contract definition: {casm_contract_path}"))?
    };

    let casm_class_hash = casm_contract_definition.class_hash()?;

    let declaration = account.declare(Arc::new(contract_definition.flatten()?), casm_class_hash);
    let execution = if let Some(max_fee) = max_fee {
        declaration.max_fee(max_fee)
    } else {
        declaration
    };
    let declared = execution.send().await;

    match declared {
        Ok(result) => {
            handle_wait_for_tx_result(account.provider(), result.transaction_hash, result).await
        }
        Err(Provider(error)) => handle_rpc_error(error),
        _ => Err(anyhow!("Unknown RPC error")),
    }
}
