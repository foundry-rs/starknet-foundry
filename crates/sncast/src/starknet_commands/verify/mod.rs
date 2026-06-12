use crate::starknet_commands::utils::felt_or_id::{ClassHash, ContractAddress};
use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, ValueEnum};
use promptly::prompt;
use scarb_metadata::PackageMetadata;
use sncast::helpers::artifacts::{contract_name_from_module_path, resolve_contract_artifacts};
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::FreeProvider;
use sncast::helpers::scarb_utils::{BuildConfig, build_and_load_artifacts};
use sncast::response::errors::StarknetCommandError;
use sncast::response::ui::UI;
use sncast::{Network, response::verify::VerifyResponse};
use sncast::{get_chain_id, get_provider};
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use std::fmt;
use url::Url;

pub mod explorer;
pub mod voyager;
pub mod walnut;

use explorer::ContractIdentifier;
use explorer::VerificationInterface;
use foundry_ui::components::warning::WarningMessage;
use voyager::Voyager;
use walnut::WalnutVerificationInterface;

#[derive(Args)]
#[command(about = "Verify a contract through a block explorer")]
pub struct Verify {
    #[command(flatten)]
    pub contract_identifier: ContractIdentifierArgs,

    /// Name of the contract or absolute module tree path
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

#[derive(Args, Clone, Debug)]
#[group(required = true, multiple = false)]
pub struct ContractIdentifierArgs {
    /// Class hash of a contract to be verified (hex, decimal, or @alias from snfoundry.toml)
    #[arg(short = 'g', long)]
    pub class_hash: Option<ClassHash>,

    /// Address of a contract to be verified (hex, decimal, or @alias from snfoundry.toml)
    #[arg(short = 'd', long)]
    pub contract_address: Option<ContractAddress>,
}

impl ContractIdentifierArgs {
    pub fn get_identifier(&self, config: &CastConfig) -> Result<ContractIdentifier> {
        match (&self.class_hash, &self.contract_address) {
            (Some(class_hash), None) => Ok(ContractIdentifier::ClassHash {
                class_hash: class_hash.resolve(config)?.to_fixed_hex_string(),
            }),
            (None, Some(contract_address)) => Ok(ContractIdentifier::Address {
                contract_address: contract_address.resolve(config)?.to_fixed_hex_string(),
            }),
            _ => unreachable!("Exactly one of class_hash or contract_address must be provided."),
        }
    }
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

fn resolve_and_validate_contract_name(
    package: &PackageMetadata,
    scarb_json: bool,
    profile: String,
    contract_name: &str,
    ui: &UI,
) -> Result<String> {
    let artifacts = build_and_load_artifacts(
        package,
        &BuildConfig {
            scarb_toml_path: package.manifest_path.clone(),
            json: scarb_json,
            profile,
        },
        false,
        // TODO(#3959) Remove `base_ui`
        ui.base_ui(),
    )
    .context("Failed to build contract")?;

    resolve_contract_artifacts(contract_name, &artifacts).map_err(|error| match error {
        StarknetCommandError::ContractArtifactsNotFound(_) => {
            anyhow!("Contract named '{contract_name}' was not found")
        }
        other => other.into(),
    })?;

    Ok(contract_name_from_module_path(contract_name).to_string())
}

fn display_files_and_confirm(
    verifier: &Verifier,
    files_to_display: Vec<String>,
    confirm_verification: bool,
    ui: &UI,
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

        if !input.is_empty() && !input.to_lowercase().starts_with('y') {
            bail!("Verification aborted");
        }
    }

    Ok(())
}

pub async fn verify(
    args: Verify,
    package: &PackageMetadata,
    json: bool,
    config: &CastConfig,
    ui: &UI,
) -> Result<VerifyResponse> {
    let Verify {
        contract_identifier,
        contract_name,
        verifier,
        network,
        confirm_verification,
        package: scarb_package,
        url,
        test_files,
    } = args;

    let rpc_url = match url {
        Some(url) => url,
        None => {
            if let Some(config_url) = config.network_params.url() {
                config_url.clone()
            } else if let Some(network) = config.network_params.network() {
                network.url(&FreeProvider::semi_random()).await?
            } else {
                let network =
                    network.ok_or_else(|| anyhow!("Either --network or --url must be provided"))?;
                network.url(&FreeProvider::semi_random()).await?
            }
        }
    };
    let provider = get_provider(&rpc_url)?;

    let workspace_dir = package
        .manifest_path
        .parent()
        .ok_or(anyhow!("Failed to obtain workspace dir"))?;

    let contract_identifier = contract_identifier.get_identifier(config)?;

    let network =
        resolve_verification_network(network, config.network_params.network(), &provider).await?;

    let resolved_contract_name = resolve_and_validate_contract_name(
        package,
        json,
        config.scarb_profile.clone(),
        &contract_name,
        ui,
    )?;

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
            display_files_and_confirm(&verifier, files_to_display, confirm_verification, ui)?;

            // Perform verification
            walnut
                .verify(
                    contract_identifier,
                    resolved_contract_name,
                    scarb_package,
                    false,
                    ui,
                )
                .await
        }
        Verifier::Voyager => {
            let voyager = Voyager::new(network, workspace_dir.to_path_buf(), &provider, ui)?;

            // Gather and format files for display
            let (_, files) = voyager.gather_files(test_files)?;
            let files_to_display: Vec<String> =
                files.keys().map(|name| format!("  {name}")).collect();

            // Display files and confirm
            display_files_and_confirm(&verifier, files_to_display, confirm_verification, ui)?;

            // Perform verification
            voyager
                .verify(
                    contract_identifier,
                    resolved_contract_name,
                    scarb_package,
                    test_files,
                    ui,
                )
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_and_validate_contract_name, resolve_verification_network};
    use camino::Utf8PathBuf;
    use scarb_metadata::PackageMetadata;
    use serde_json::json;
    use sncast::Network;
    use sncast::get_provider;
    use sncast::helpers::scarb_utils::get_package_metadata;
    use sncast::response::ui::UI;
    use starknet_types_core::felt::Felt;
    use url::Url;
    use wiremock::matchers::{body_partial_json, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn unused_provider() -> starknet_rust::providers::jsonrpc::JsonRpcClient<
        starknet_rust::providers::jsonrpc::HttpTransport,
    > {
        get_provider(&Url::parse("http://127.0.0.1:1").unwrap()).unwrap()
    }

    fn package_metadata_for_test_contract(contract_dir: &str) -> PackageMetadata {
        let manifest_path = Utf8PathBuf::from(format!(
            "{}/tests/data/contracts/{contract_dir}/Scarb.toml",
            env!("CARGO_MANIFEST_DIR")
        ));

        get_package_metadata(&manifest_path, &None).unwrap()
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

    #[test]
    fn resolves_contract_name_from_full_module_path() {
        let package = package_metadata_for_test_contract("duplicate_contract_name");
        let ui = UI::default();

        let contract_name = resolve_and_validate_contract_name(
            &package,
            false,
            "release".to_string(),
            "duplicate_contract_name::first_contract::HelloStarknet",
            &ui,
        )
        .unwrap();

        assert_eq!(contract_name, "HelloStarknet");
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
