use anyhow::{anyhow, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use cast::helpers::{response_structs::DeclareResponse, scarb_utils::get_scarb_manifest};
use cast::{handle_rpc_error, handle_wait_for_tx};
use clap::Args;
use starknet::accounts::AccountError::Provider;
use starknet::accounts::ConnectedAccount;
use starknet::core::types::FieldElement;
use starknet::{
    accounts::{Account, SingleOwnerAccount},
    core::types::contract::{CompiledClass, SierraClass},
    providers::jsonrpc::{HttpTransport, JsonRpcClient},
    signers::LocalWallet,
};
use starknet_artifacts::{get_contracts_map, try_get_starknet_artifacts_path};
use std::process::{Command, Stdio};
use std::sync::Arc;

#[derive(Args)]
#[command(about = "Declare a contract to starknet", long_about = None)]
pub struct Declare {
    /// Contract name
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
    account: &SingleOwnerAccount<&JsonRpcClient<HttpTransport>, LocalWallet>,
    path_to_scarb_toml: &Option<Utf8PathBuf>,
    wait: bool,
) -> Result<DeclareResponse> {
    let contract_name: String = contract_name.to_string();
    let manifest_path = match path_to_scarb_toml.clone() {
        Some(path) => path,
        None => get_scarb_manifest().context("Failed to obtain manifest path from scarb")?,
    };

    let command_result = Command::new("scarb")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--release")
        .arg("build")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("Failed to start building contracts with Scarb")?;
    let result_code = command_result
        .status
        .code()
        .context("Failed to obtain status code from scarb build")?;
    let result_msg = String::from_utf8(command_result.stdout)?;
    if result_code != 0 {
        anyhow::bail!(
            "Scarb build returned non-zero exit code: {} - error message: {}",
            result_code,
            result_msg
        );
    }

    let metadata = scarb_metadata::MetadataCommand::new()
        .inherit_stderr()
        .exec()
        .context("Failed to obtain scarb metadata")?;

    // TODO #41 improve handling starknet artifacts

    let compiled_directory = metadata
        .target_dir
        .map(|path| path.join("release"))
        .ok_or(anyhow!("Failed to obtain path to compiled contracts"))?;

    let mut paths = compiled_directory
        .read_dir()
        .context("Failed to read ./target/release, scarb build probably failed")?;

    let starknet_artifacts = paths
        .find_map(|path| {
            path.ok().and_then(|entry| {
                let name = entry.file_name().into_string().ok()?;
                name.contains("starknet_artifacts").then_some(entry.path())
            })
        })
        .ok_or(anyhow!("Failed to find starknet_artifacts.json file"))?;

    // let artifacts_path = try_get_starknet_artifacts_path(&compiled_directory, &contract_name)?
    //     .ok_or(anyhow!("Failed to find starknet_artifacts.json file"))?;

    let artifacts_path = Utf8Path::from_path(&starknet_artifacts).ok_or(anyhow!("xyz"))?;

    let contracts = get_contracts_map(artifacts_path)?;
    let contract_artifacts = contracts
        .get(&contract_name)
        .ok_or(anyhow!("Failed to find artifacts in starknet_artifacts.json file. Make sure you have enabled sierra and casm code generation in Scarb.toml"))?;
    // let contracts_map = get_contracts_map(&Utf8Path::from(starknet_artifacts))?;

    // let starknet_artifacts = std::fs::read_to_string(&starknet_artifacts)
    //     .with_context(|| format!("Failed to read {starknet_artifacts:?} contents"))?;

    // let starknet_artifacts: StarknetArtifacts =
    //     serde_json::from_str(starknet_artifacts.as_str())
    //         .context("Failed to parse starknet_artifacts.json contents")?;

    // let sierra_path = starknet_artifacts
    //     .contracts
    //     .iter()
    //     .find_map(|contract| {
    //         if contract.contract_name == contract_name {
    //             return contract.artifacts.sierra.clone();
    //         }
    //         None
    //     })
    //     .ok_or(anyhow!("Cannot find sierra artifact {contract_name} in starknet_artifacts.json - please make sure sierra is set to 'true' under your [[target.starknet-contract]] field in Scarb.toml"))?;

    // let casm_path = starknet_artifacts
    //     .contracts
    //     .iter()
    //     .find_map(|contract| {
    //         if contract.contract_name == contract_name {
    //             return contract.artifacts.casm.clone();
    //         }
    //         None
    //     })
    //     .ok_or(anyhow!("Cannot find casm artifact {contract_name} in starknet_artifacts.json - please make sure casm is set to 'true' under your [[target.starknet-contract]] field in Scarb.toml"))?;

    // let sierra_contract_path = &compiled_directory.join(sierra_path);
    // let casm_contract_path = &compiled_directory.join(casm_path);

    let contract_definition: SierraClass = {
        // let file_contents = std::fs::read(sierra_contract_path.clone())
        //     .with_context(|| format!("Failed to read contract file: {sierra_contract_path}"))?;
        serde_json::from_str(&contract_artifacts.sierra)
            .with_context(|| "Failed to parse sierra artifact")?
    };
    let casm_contract_definition: CompiledClass = {
        // let file_contents = std::fs::read(casm_contract_path.clone())
        // .with_context(|| format!("Failed to read contract file: {casm_contract_path}"))?;
        serde_json::from_str(&contract_artifacts.casm)
            .with_context(|| "Failed to parse casm artifact")?
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
            handle_wait_for_tx(
                account.provider(),
                result.transaction_hash,
                DeclareResponse {
                    class_hash: result.class_hash,
                    transaction_hash: result.transaction_hash,
                },
                wait,
            )
            .await
        }
        Err(Provider(error)) => handle_rpc_error(error),
        _ => Err(anyhow!("Unknown RPC error")),
    }
}
