use anyhow::{Result, anyhow, bail};
use camino::Utf8PathBuf;
use clap::{ArgGroup, Args, ValueEnum};
use promptly::prompt;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::FreeProvider;
use sncast::response::ui::UI;
use sncast::{Network, response::verify::VerifyResponse};
use sncast::{get_chain_id, get_provider};
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use starknet_types_core::felt::Felt;
use std::{collections::HashMap, fmt};
use url::Url;

pub mod explorer;
pub mod voyager;
pub mod walnut;

use explorer::ContractIdentifier;
use explorer::VerificationInterface;
use foundry_ui::components::warning::WarningMessage;
use sncast::helpers::artifacts::CastStarknetContractArtifacts;
use voyager::Voyager;
use walnut::WalnutVerificationInterface;

#[derive(Args)]
#[command(about = "Verify a contract through a block explorer")]
#[command(group(
    ArgGroup::new("contract_identifier")
        .required(true)
        .args(&["class_hash", "contract_address"])
))]
pub struct Verify {
    /// Class hash of a contract to be verified
    #[arg(short = 'g', long)]
    pub class_hash: Option<Felt>,

    /// Address of a contract to be verified
    #[arg(short = 'd', long)]
    pub contract_address: Option<Felt>,

    /// Name of the contract that is being verified
    #[arg(short, long)]
    pub contract_name: String,

    /// Block explorer to use for the verification
    #[arg(short, long, value_enum)]
    pub verifier: Verifier,

    /// The network on which block explorer will do the verification
    #[arg(short, long, value_enum)]
    pub network: Option<Network>,

    /// Assume "yes" as answer to confirmation prompt and run non-interactively
    #[arg(long, default_value = "false")]
    pub confirm_verification: bool,

    /// Specifies scarb package to be used
    #[arg(long)]
    pub package: Option<String>,

    /// RPC provider url address; overrides url from snfoundry.toml. Will use public provider if not set.
    #[arg(long)]
    pub url: Option<Url>,

    /// Include test files under src/ for verification (only applies to voyager)
    #[arg(long, default_value = "false")]
    pub test_files: bool,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Verifier {
    Walnut,
    Voyager,
}

impl fmt::Display for Verifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Verifier::Walnut => write!(f, "walnut"),
            Verifier::Voyager => write!(f, "voyager"),
        }
    }
}

async fn resolve_verification_network(
    cli_network: Option<Network>,
    config_network: Option<Network>,
    provider: &JsonRpcClient<HttpTransport>,
) -> Result<Network> {
    if let Some(network) = cli_network.or(config_network) {
        return Ok(network);
    }

    let chain_id = get_chain_id(provider).await?;

    Network::try_from(chain_id).map_err(|_| {
            anyhow!(
                "Failed to infer verification network from the RPC chain ID {chain_id:#x}; pass `--network mainnet` or `--network sepolia` explicitly"
            )
        })
}

fn display_files_and_confirm(
    verifier: &Verifier,
    files_to_display: Vec<String>,
    confirm_verification: bool,
    ui: &UI,
    artifacts: &HashMap<String, CastStarknetContractArtifacts>,
    contract_name: &str,
) -> Result<()> {
    // Display files that will be uploaded
    // TODO(#3960) JSON output support
    ui.print_notification("The following files will be uploaded to verifier:".to_string());
    for file_path in files_to_display {
        ui.print_notification(file_path);
    }

    // Ask for confirmation after showing files
    if !confirm_verification {
        let prompt_text = format!(
            "\n\tYou are about to submit the above files to the third-party verifier at {verifier}.\n\n\tImportant: Make sure your project's Scarb.toml does not include sensitive information like private keys.\n\n\tAre you sure you want to proceed? (Y/n)"
        );
        let input: String = prompt(prompt_text)?;

        if !input.to_lowercase().starts_with('y') {
            bail!("Verification aborted");
        }
    }

    // Check contract exists after confirmation
    if !artifacts.contains_key(contract_name) {
        return Err(anyhow!("Contract named '{contract_name}' was not found"));
    }

    Ok(())
}

pub async fn verify(
    args: Verify,
    manifest_path: &Utf8PathBuf,
    artifacts: &HashMap<String, CastStarknetContractArtifacts>,
    config: &CastConfig,
    ui: &UI,
) -> Result<VerifyResponse> {
    let Verify {
        contract_address,
        class_hash,
        contract_name,
        verifier,
        network,
        confirm_verification,
        package,
        url,
        test_files,
    } = args;

    let rpc_url = match url {
        Some(url) => url,
        None => {
            if let Some(config_url) = &config.network_params.url {
                config_url.clone()
            } else if let Some(network) = &config.network_params.network {
                network.url(&FreeProvider::semi_random()).await?
            } else {
                let network =
                    network.ok_or_else(|| anyhow!("Either --network or --url must be provided"))?;
                network.url(&FreeProvider::semi_random()).await?
            }
        }
    };
    let provider = get_provider(&rpc_url)?;

    // Build JSON Payload for the verification request
    // get the parent dir of the manifest path
    let workspace_dir = manifest_path
        .parent()
        .ok_or(anyhow!("Failed to obtain workspace dir"))?;

    let contract_identifier = match (class_hash, contract_address) {
        (Some(class_hash), None) => ContractIdentifier::ClassHash {
            class_hash: class_hash.to_fixed_hex_string(),
        },
        (None, Some(contract_address)) => ContractIdentifier::Address {
            contract_address: contract_address.to_fixed_hex_string(),
        },

        _ => {
            unreachable!("Exactly one of class_hash or contract_address must be provided.");
        }
    };

    let network = resolve_verification_network(network, config.network_params.network, &provider).await?;

    // Handle test_files warning for Walnut
    if matches!(verifier, Verifier::Walnut) && test_files {
        ui.print_warning(WarningMessage::new(
            "The `--test-files` option is ignored for Walnut verifier",
        ));
    }

    // Create verifier instance, gather files, and perform verification
    match verifier {
        Verifier::Walnut => {
            let walnut = WalnutVerificationInterface::new(
                network,
                workspace_dir.to_path_buf(),
                &provider,
                ui,
            )?;

            // Gather and format files for display
            let files = walnut.gather_files()?;
            let files_to_display: Vec<String> =
                files.iter().map(|(path, _)| format!("  {path}")).collect();

            // Display files and confirm
            display_files_and_confirm(
                &verifier,
                files_to_display,
                confirm_verification,
                ui,
                artifacts,
                &contract_name,
            )?;

            // Perform verification
            walnut
                .verify(contract_identifier, contract_name, package, false, ui)
                .await
        }
        Verifier::Voyager => {
            let voyager = Voyager::new(network, workspace_dir.to_path_buf(), &provider, ui)?;

            // Gather and format files for display
            let (_, files) = voyager.gather_files(test_files)?;
            let files_to_display: Vec<String> =
                files.keys().map(|name| format!("  {name}")).collect();

            // Display files and confirm
            display_files_and_confirm(
                &verifier,
                files_to_display,
                confirm_verification,
                ui,
                artifacts,
                &contract_name,
            )?;

            // Perform verification
            voyager
                .verify(contract_identifier, contract_name, package, test_files, ui)
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_verification_network;
    use serde_json::json;
    use sncast::Network;
    use sncast::get_provider;
    use starknet_types_core::felt::Felt;
    use url::Url;
    use wiremock::matchers::{body_partial_json, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn unused_provider() -> starknet_rust::providers::jsonrpc::JsonRpcClient<
        starknet_rust::providers::jsonrpc::HttpTransport,
    > {
        get_provider(&Url::parse("http://127.0.0.1:1").unwrap()).unwrap()
    }

    async fn mock_provider_for_chain_id(
        chain_id: Felt,
    ) -> (
        starknet_rust::providers::jsonrpc::JsonRpcClient<
            starknet_rust::providers::jsonrpc::HttpTransport,
        >,
        MockServer,
    ) {
        let mock_rpc = MockServer::start().await;
        Mock::given(method("POST"))
            .and(body_partial_json(json!({"method": "starknet_chainId"})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 1,
                "jsonrpc": "2.0",
                "result": format!("{chain_id:#x}")
            })))
            .expect(1)
            .mount(&mock_rpc)
            .await;

        (
            get_provider(&Url::parse(&mock_rpc.uri()).unwrap()).unwrap(),
            mock_rpc,
        )
    }

    #[tokio::test]
    async fn uses_cli_network_when_provided() {
        let provider = unused_provider();
        let network =
            resolve_verification_network(Some(Network::Mainnet), Some(Network::Sepolia), &provider)
                .await
                .unwrap();

        assert_eq!(network, Network::Mainnet);
    }

    #[tokio::test]
    async fn uses_config_network_when_cli_network_is_missing() {
        let provider = unused_provider();
        let network = resolve_verification_network(None, Some(Network::Mainnet), &provider)
            .await
            .unwrap();

        assert_eq!(network, Network::Mainnet);
    }

    #[tokio::test]
    async fn infers_mainnet_from_chain_id_when_no_network_is_configured() {
        let (provider, _mock_rpc) = mock_provider_for_chain_id(sncast::MAINNET).await;
        let network = resolve_verification_network(None, None, &provider)
            .await
            .unwrap();

        assert_eq!(network, Network::Mainnet);
    }

    #[tokio::test]
    async fn infers_sepolia_from_chain_id_when_no_network_is_configured() {
        let (provider, _mock_rpc) = mock_provider_for_chain_id(sncast::SEPOLIA).await;
        let network = resolve_verification_network(None, None, &provider)
            .await
            .unwrap();

        assert_eq!(network, Network::Sepolia);
    }

    #[tokio::test]
    async fn errors_when_network_cannot_be_resolved() {
        let (provider, _mock_rpc) =
            mock_provider_for_chain_id(Felt::from_hex_unchecked("0x1234")).await;
        let error = resolve_verification_network(None, None, &provider)
            .await
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Failed to infer verification network from the RPC chain ID 0x1234")
        );
    }
}
