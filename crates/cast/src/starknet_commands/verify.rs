use anyhow::Ok;
use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use cast::helpers::response_structs::VerificationStatus;
use cast::helpers::response_structs::VerifyResponse;
use cast::helpers::scarb_utils::get_scarb_manifest;
use clap::Args;
use serde::Serialize;
use starknet::core::types::FieldElement;
use walkdir::WalkDir;

struct VoyagerVerificationInterface {
    network: String,
    workspace_dir: Utf8PathBuf,
}

struct StarkscanVerificationInterface {
    network: String,
    workspace_dir: Utf8PathBuf,
}

#[async_trait::async_trait]
trait VerificationInterface {
    fn new(network: String, workspace_dir: Utf8PathBuf) -> Self;
    async fn verify(
        &self,
        contract_address: FieldElement,
        contract_name: String,
    ) -> Result<VerifyResponse>;
    fn gen_explorer_url(&self) -> String;
}

#[async_trait::async_trait]
impl VerificationInterface for VoyagerVerificationInterface {
    fn new(network: String, workspace_dir: Utf8PathBuf) -> Self {
        VoyagerVerificationInterface {
            network,
            workspace_dir,
        }
    }

    async fn verify(
        &self,
        contract_address: FieldElement,
        contract_name: String,
    ) -> Result<VerifyResponse> {
        let verification_status = VerificationStatus::OK;
        let errors = None;

        // create a dummy response for now until the Voyager APIs come online
        let res = match verification_status {
            VerificationStatus::OK => Ok(VerifyResponse {
                verification_status,
                errors,
            }),
            VerificationStatus::Error => Err(anyhow!("Unknown RPC error")),
        };

        // Read all files name along with their contents in a JSON format 
        // in the workspace dir recursively
        // key is the file name and value is the file content

        let mut file_data = serde_json::Map::new();

        // Recursively read files and their contents in workspace directory
        for entry in WalkDir::new(self.workspace_dir.clone()).follow_links(true) {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && (path.ends_with(".cairo") || path.ends_with(".toml")) {
                let file_name = path.file_name().unwrap().to_string_lossy().to_string();
                let file_content = std::fs::read_to_string(path)?;
                file_data.insert(file_name, serde_json::Value::String(file_content));
            }
        }

        // Serialize the JSON object to a JSON string
        let source_code = serde_json::Value::Object(file_data);

        // Create the JSON payload with "contract name," "address," and "source_code" fields
        let payload = VerificationPayload {
            contract_name: contract_name.to_string(),
            contract_address: contract_address.to_string(),
            source_code,
        };

        // Serialize the payload to a JSON string for the POST request
        let json_payload = serde_json::to_string(&payload)?;

        // Send the POST request to the explorer
        let client = reqwest::Client::new();
        let api_res = client
            .post(self.gen_explorer_url())
            .body(json_payload)
            .send()
            .await?
            .text()
            .await?;

        // Parse the response from the explorer
        // let api_res: serde_json::Value = serde_json::from_str(&api_res)?;
        println!("{api_res:?}");

        res
    }

    fn gen_explorer_url(&self) -> String {
        let explorer_url: &str = match self.network.as_str() {
            "mainnet" => "https://voyager.online/",
            _ => "https://goerli.voyager.online/",
        };
        explorer_url.to_string()
    }
}

#[async_trait::async_trait]
impl VerificationInterface for StarkscanVerificationInterface {
    fn new(network: String, workspace_dir: Utf8PathBuf) -> Self {
        StarkscanVerificationInterface {
            network,
            workspace_dir,
        }
    }

    async fn verify(
        &self,
        _contract_address: FieldElement,
        _contract_name: String,
    ) -> Result<VerifyResponse> {
        // This function left unimplemented for now
        // It will be implemented once the starkscan explorer APIs are ready

        println!(
            "Verifying {workspace_dir:?} workspace",
            workspace_dir = self.workspace_dir
        );

        let verification_status = VerificationStatus::OK;
        let errors = None;

        // create a dummy response for now until the Starkscan APIs come online
        match verification_status {
            VerificationStatus::OK => Ok(VerifyResponse {
                verification_status,
                errors,
            }),
            VerificationStatus::Error => Err(anyhow!("Unknown RPC error")),
        }
    }

    fn gen_explorer_url(&self) -> String {
        let explorer_url: &str = match self.network.as_str() {
            "mainnet" => "https://starkscan.co/",
            _ => "https://testnet.starkscan.co/",
        };
        explorer_url.to_string()
    }
}

#[derive(Args)]
#[command(about = "Verify a contract through a block explorer")]
pub struct Verify {
    /// Address of a contract to be verified
    #[clap(short = 'a', long)]
    pub contract_address: FieldElement,

    /// Name of the contract that is being verified
    #[clap(short, long)]
    pub contract_name: String,

    /// Block explorer to use for the verification
    #[clap(short = 'v', long = "verifier", value_parser = ["voyager", "starkscan"])]
    pub verifier: String,

    /// The network on which block explorer will do the verification
    #[clap(short = 'n', long = "network", value_parser = ["mainnet", "goerli"])]
    pub network: String,
}

#[derive(Serialize, Debug)]
struct VerificationPayload {
    contract_name: String,
    contract_address: String,
    source_code: serde_json::Value,
}

pub async fn verify(
    contract_address: FieldElement,
    contract_name: String,
    verifier: String,
    network: String,
    path_to_scarb_toml: &Option<Utf8PathBuf>,
) -> Result<VerifyResponse> {
    let manifest_path: Utf8PathBuf = match path_to_scarb_toml.clone() {
        Some(path) => path,
        None => get_scarb_manifest().context("Failed to obtain manifest path from scarb")?,
    };

    // Build JSON Payload for the verification request
    // get the parent dir of the manifest path
    let workspace_dir = manifest_path
        .parent()
        .ok_or(anyhow!("Failed to obtain workspace dir"))?;

    match verifier.as_str() {
        "voyager" => {
            let voyager = VoyagerVerificationInterface::new(network, workspace_dir.to_path_buf());
            voyager
                .verify(contract_address, contract_name)
                .await
                .context("Failed to verify contract")
        }
        "starkscan" => {
            let starkscan =
                StarkscanVerificationInterface::new(network, workspace_dir.to_path_buf());
            starkscan
                .verify(contract_address, contract_name)
                .await
                .context("Failed to verify contract")
        }
        _ => Err(anyhow!("Unknown verifier")),
    }
}
