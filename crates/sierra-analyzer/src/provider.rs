use reqwest::{Client, Error};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Configuration for endpoints
/// TODO : Add other endpoints in case of failure
pub struct NetworkConfig;

impl NetworkConfig {
    /// URL for the mainnet API
    pub const MAINNET_API_URL: &'static str =
        "https://starknet-mainnet.public.blastapi.io/rpc/v0_7";

    /// URL for the Sepolia API
    pub const SEPOLIA_API_URL: &'static str =
        "https://starknet-sepolia.public.blastapi.io/rpc/v0_7";
}

/// Struct representing an RPC client
pub struct RpcClient<'a> {
    /// The client used for sending requests
    client: Client,
    /// The address of the RPC node
    node_address: &'a str,
}

impl<'a> RpcClient<'a> {
    pub fn new(node_address: &'a str) -> Self {
        RpcClient {
            client: Client::new(),
            node_address,
        }
    }

    /// Sends a starknet_getClass request to the RPC node.
    pub async fn get_class(&self, contract_class: &str) -> Result<RpcResponse, Error> {
        let url = format!("{}", self.node_address);
        let request_body = serde_json::json!({
            "id": 1,
            "jsonrpc": "2.0",
            "method": "starknet_getClass",
            "params": ["pending", contract_class],
        });

        let response = self
            .client
            .post(&url)
            .json(&request_body)
            .send()
            .await?
            .json::<RpcResponse>()
            .await?;

        Ok(response)
    }
}

/// The expected response structure from the starknet_getClass RPC call
#[derive(Deserialize, Serialize, Debug)]
pub struct RpcResponse {
    pub result: Value,
}

impl RpcResponse {
    /// Returns the response JSON
    pub fn to_json(&self) -> String {
        // Serialize the RpcResponse into a JSON string
        let json_string = serde_json::to_string_pretty(&self.result)
            .unwrap_or_else(|e| format!("Error serializing JSON: {}", e));

        // Parse the JSON string into a serde_json::Value
        let value: Value = serde_json::from_str(&json_string)
            .unwrap_or_else(|e| panic!("Error parsing JSON: {}", e));

        // Clean the ABI field
        let clean_abi = value["abi"]
            .as_str()
            .unwrap_or_else(|| panic!("Missing ABI field"))
            .replace(r#"\""#, "") // Remove escaped quotes
            .trim_matches('"') // Trim surrounding quotes
            .to_string();

        // Extract other fields
        let sierra_program = &value["sierra_program"].to_string();
        let sierra_program_debug_info = &value["sierra_program_debug_info"].to_string();
        let contract_class_version = &value["contract_class_version"].to_string();
        let entry_points_by_type = &value["entry_points_by_type"].to_string();

        // Construct the formatted JSON string
        let sierra_json = format!(
            r#"{{
    "abi": {},
    "sierra_program": {},
    "sierra_program_debug_info": {},
    "contract_class_version": {},
    "entry_points_by_type": {}
}}"#,
            clean_abi,
            sierra_program,
            sierra_program_debug_info,
            contract_class_version,
            entry_points_by_type
        )
        .trim() // Trim leading and trailing whitespaces
        .to_string();

        sierra_json
    }
}
