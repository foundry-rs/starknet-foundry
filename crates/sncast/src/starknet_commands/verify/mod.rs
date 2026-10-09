use crate::starknet_commands::utils::felt_or_id::{ClassHash, ContractAddress};
use anyhow::{Context, Result, anyhow, bail};
use camino::Utf8PathBuf;
use clap::{Args, ValueEnum};
use foundry_ui::OutputFormat;
use promptly::prompt_default;
use scarb_metadata::PackageMetadata;
use shared::utils::contract_name_from_module_path;
use sncast::Network;
use sncast::get_chain_id;
use sncast::helpers::artifacts::resolve_contract_artifacts;
use sncast::helpers::command::process_command_result;
use sncast::helpers::configuration::CastConfig;
use sncast::helpers::rpc::RpcArgs;
use sncast::helpers::scarb_utils::{
    BuildConfig, assert_manifest_path_exists, build_and_load_artifacts, get_package_metadata,
};
use sncast::response::errors::StarknetCommandError;
use sncast::response::ui::UI;
use sncast::response::verify::VerifyResponse;
use starknet_rust::providers::jsonrpc::{HttpTransport, JsonRpcClient};
use std::fmt;
use std::process::ExitCode;

pub mod explorer;
pub mod starkloupe;
pub mod voyager;

use explorer::ContractIdentifier;
use explorer::VerificationInterface;
use foundry_ui::components::warning::WarningMessage;
use starkloupe::StarkloupeVerificationInterface;
use voyager::Voyager;

#[derive(Args)]
#[command(about = "Verify a contract through a block explorer")]
pub struct Verify {
    #[command(flatten)]
    pub contract_identifier: ContractIdentifierArgs,

    #[command(flatten)]
    pub rpc: RpcArgs,

    #[command(flatten)]
    pub common: VerifyCommonArgs,

    /// Name of the contract or module tree path suffix
    #[arg(short, long)]
    pub contract_name: String,

    /// Specifies scarb package to be used
    #[arg(long)]
    pub package: Option<String>,
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

#[derive(Args)]
pub struct VerifyCommonArgs {
    /// Block explorer to use for the verification [possible values: starkloupe, voyager]
    #[arg(short, long, value_parser = parse_verifier)]
    pub verifier: Verifier,

    /// Include test files under src/ for verification (only applies to voyager)
    #[arg(long, default_value = "false", requires = "verifier")]
    pub test_files: bool,

    /// Assume "yes" as answer to confirmation prompt and run non-interactively
    #[arg(long, default_value = "false", requires = "verifier")]
    pub confirm_verification: bool,
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
    Starkloupe,
    Voyager,
}

impl fmt::Display for Verifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Verifier::Starkloupe => write!(f, "starkloupe"),
            Verifier::Voyager => write!(f, "voyager"),
        }
    }
}

fn parse_verifier(value: &str) -> Result<Verifier, String> {
    if value.eq_ignore_ascii_case("walnut") {
        return Err(
            "Walnut has been migrated to Starkloupe; use `--verifier starkloupe` instead. See https://app.starkloupe.co for more information."
                .to_string(),
        );
    }
    <Verifier as ValueEnum>::from_str(value, true)
}

pub async fn resolve_verification_network(
    rpc: &RpcArgs,
    provider: &JsonRpcClient<HttpTransport>,
    config: &CastConfig,
) -> Result<Network> {
    let network = rpc
        .get_network_params(config)
        .ok()
        .and_then(|p| p.network());

    if let Some(network) = network {
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
    profile: String,
    contract_name: &str,
    ui: &UI,
) -> Result<String> {
    // TODO(#3959) Remove `base_ui`
    let base_ui = ui.base_ui();
    let artifacts = build_and_load_artifacts(
        package,
        &BuildConfig {
            scarb_toml_path: package.manifest_path.clone(),
            json: base_ui.output_format() == OutputFormat::Json,
            profile,
        },
        base_ui,
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
            "\n\tYou are about to submit the above files to the third-party verifier at {verifier}.\n\n\tImportant: Make sure your project's Scarb.toml does not include sensitive information like private keys.\n\n\tAre you sure you want to proceed?"
        );

        if !prompt_default(prompt_text, true)? {
            bail!("Verification aborted")
        }
    }

    Ok(())
}

pub async fn verify(args: Verify, config: CastConfig, ui: &UI) -> Result<ExitCode> {
    let Verify {
        contract_identifier,
        rpc,
        common,
        contract_name,
        package: scarb_package,
    } = args;

    let manifest_path = assert_manifest_path_exists()?;
    let package = get_package_metadata(&manifest_path, &scarb_package)?;

    let provider = rpc.get_provider(&config, ui).await?;

    let workspace_dir = package
        .manifest_path
        .parent()
        .ok_or(anyhow!("Failed to obtain workspace dir"))?;

    let contract_identifier = contract_identifier.get_identifier(&config)?;

    let network = resolve_verification_network(&rpc, &provider, &config).await?;

    let resolved_contract_name = resolve_and_validate_contract_name(
        &package,
        config.scarb_profile.clone(),
        &contract_name,
        ui,
    )?;

    let result = verify_contract(
        common,
        contract_identifier,
        resolved_contract_name,
        scarb_package,
        &provider,
        network,
        workspace_dir.to_path_buf(),
        ui,
    )
    .await;

    Ok(process_command_result("verify", result, ui, None))
}

#[expect(clippy::too_many_arguments)]
pub async fn verify_contract(
    common: VerifyCommonArgs,
    contract_identifier: ContractIdentifier,
    contract_name: String,
    scarb_package: Option<String>,
    provider: &JsonRpcClient<HttpTransport>,
    network: Network,
    workspace_dir: Utf8PathBuf,
    ui: &UI,
) -> Result<VerifyResponse> {
    // Handle test_files warning for Starkloupe
    if matches!(common.verifier, Verifier::Starkloupe) && common.test_files {
        ui.print_warning(WarningMessage::new(
            "The `--test-files` option is ignored for Starkloupe verifier",
        ));
    }

    // Create verifier instance, gather files, and perform verification
    match common.verifier {
        Verifier::Starkloupe => {
            let starkloupe =
                StarkloupeVerificationInterface::new(network, workspace_dir, provider, ui)?;

            // Gather and format files for display
            let files = starkloupe.gather_files()?;
            let files_to_display: Vec<String> =
                files.iter().map(|(path, _)| format!("  {path}")).collect();

            // Display files and confirm
            display_files_and_confirm(
                &common.verifier,
                files_to_display,
                common.confirm_verification,
                ui,
            )?;

            // Perform verification
            starkloupe
                .verify(contract_identifier, contract_name, scarb_package, false, ui)
                .await
        }
        Verifier::Voyager => {
            let voyager = Voyager::new(network, workspace_dir, provider, ui)?;

            // Gather and format files for display
            let (_, files) = voyager.gather_files(common.test_files)?;
            let files_to_display: Vec<String> =
                files.keys().map(|name| format!("  {name}")).collect();

            // Display files and confirm
            display_files_and_confirm(
                &common.verifier,
                files_to_display,
                common.confirm_verification,
                ui,
            )?;

            // Perform verification
            voyager
                .verify(
                    contract_identifier,
                    contract_name,
                    scarb_package,
                    common.test_files,
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
    use sncast::helpers::configuration::CastConfig;
    use sncast::helpers::rpc::RpcArgs;
    use sncast::helpers::scarb_utils::get_package_metadata;
    use sncast::response::ui::UI;
    use starknet_types_core::felt::Felt;
    use url::Url;
    use wiremock::matchers::{body_partial_json, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

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
            "release".to_string(),
            "duplicate_contract_name::first_contract::HelloStarknet",
            &ui,
        )
        .unwrap();

        assert_eq!(contract_name, "HelloStarknet");
    }

    #[test]
    fn resolves_contract_name_from_partial_module_path() {
        let package = package_metadata_for_test_contract("duplicate_contract_name");
        let ui = UI::default();

        let contract_name = resolve_and_validate_contract_name(
            &package,
            "release".to_string(),
            "first_contract::HelloStarknet",
            &ui,
        )
        .unwrap();

        assert_eq!(contract_name, "HelloStarknet");
    }

    #[tokio::test]
    async fn infers_mainnet_from_chain_id_when_no_network_is_configured() {
        let (provider, _mock_rpc) = mock_provider_for_chain_id(sncast::MAINNET).await;
        let network =
            resolve_verification_network(&RpcArgs::default(), &provider, &CastConfig::default())
                .await
                .unwrap();

        assert_eq!(network, Network::Mainnet);
    }

    #[tokio::test]
    async fn infers_sepolia_from_chain_id_when_no_network_is_configured() {
        let (provider, _mock_rpc) = mock_provider_for_chain_id(sncast::SEPOLIA).await;
        let network =
            resolve_verification_network(&RpcArgs::default(), &provider, &CastConfig::default())
                .await
                .unwrap();

        assert_eq!(network, Network::Sepolia);
    }

    #[tokio::test]
    async fn errors_when_network_cannot_be_resolved() {
        let (provider, _mock_rpc) =
            mock_provider_for_chain_id(Felt::from_hex_unchecked("0x1234")).await;
        let error =
            resolve_verification_network(&RpcArgs::default(), &provider, &CastConfig::default())
                .await
                .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Failed to infer verification network from the RPC chain ID 0x1234")
        );
    }
}
