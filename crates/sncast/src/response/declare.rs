use super::{command::CommandResponse, explorer_link::OutputLink};
use crate::Network;
use crate::helpers::block_explorer::LinkProvider;
use crate::helpers::rpc::generate_network_flag;
use crate::response::cast_message::SncastMessage;
use anyhow::Error;
use camino::Utf8PathBuf;
use conversions::string::IntoHexStr;
use conversions::{padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::Message;
use foundry_ui::styling;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use starknet::core::types::contract::{AbiConstructor, AbiEntry};
use std::fmt::Write;
#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct DeclareTransactionResponse {
    pub class_hash: PaddedFelt,
    pub transaction_hash: PaddedFelt,
}

impl CommandResponse for DeclareTransactionResponse {}

impl Message for SncastMessage<DeclareTransactionResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Declaration completed")
            .blank_line()
            .field(
                "Class Hash",
                &self.command_response.class_hash.into_hex_string(),
            )
            .field(
                "Transaction Hash",
                &self.command_response.transaction_hash.into_hex_string(),
            )
            .build()
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.command_response).unwrap_or_else(|err| {
            json!({
                "error": "Failed to serialize response",
                "command": self.command,
                "details": err.to_string()
            })
        })
    }
}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct AlreadyDeclaredResponse {
    pub class_hash: PaddedFelt,
}

impl CommandResponse for AlreadyDeclaredResponse {}

impl Message for SncastMessage<AlreadyDeclaredResponse> {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Contract class already declared")
            .blank_line()
            .field(
                "Class Hash",
                &self.command_response.class_hash.into_hex_string(),
            )
            .build()
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.command_response).unwrap_or_else(|err| {
            json!({
                "error": "Failed to serialize response",
                "command": self.command,
                "details": err.to_string()
            })
        })
    }
}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
#[serde(tag = "status")]
pub enum DeclareResponse {
    AlreadyDeclared(AlreadyDeclaredResponse),
    #[serde(untagged)]
    Success(DeclareTransactionResponse),
}

impl CommandResponse for DeclareResponse {}

impl Message for SncastMessage<DeclareResponse> {
    fn text(&self) -> String {
        match &self.command_response {
            DeclareResponse::AlreadyDeclared(response) => styling::OutputBuilder::new()
                .success_message("Contract class already declared")
                .blank_line()
                .field("Class Hash", &response.class_hash.into_hex_string())
                .build(),
            DeclareResponse::Success(response) => styling::OutputBuilder::new()
                .success_message("Declaration completed")
                .blank_line()
                .field("Class Hash", &response.class_hash.into_hex_string())
                .field(
                    "Transaction Hash",
                    &response.transaction_hash.into_hex_string(),
                )
                .build(),
        }
    }

    fn json(&self) -> Value {
        serde_json::to_value(&self.command_response).unwrap_or_else(|err| {
            json!({
                "error": "Failed to serialize response",
                "command": self.command,
                "details": err.to_string()
            })
        })
    }
}

impl OutputLink for DeclareTransactionResponse {
    const TITLE: &'static str = "declaration";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        formatdoc!(
            "
            class: {}
            transaction: {}
            ",
            provider.class(self.class_hash),
            provider.transaction(self.transaction_hash)
        )
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct DeployCommandMessage {
    accounts_file: Option<String>,
    account: String,
    class_hash: PaddedFelt,
    arguments: Option<String>,
    network_flag: String,
}

impl DeployCommandMessage {
    pub fn new(
        abi: &[AbiEntry],
        response: &DeclareTransactionResponse,
        account: &str,
        accounts_file: &Utf8PathBuf,
        rpc_url: Option<&str>,
        network: Option<&Network>,
    ) -> Result<Self, Error> {
        let arguments = abi.iter().find_map(|entry| {
            if let AbiEntry::Constructor(constructor) = entry {
                let args = generate_constructor_placeholder_args(constructor.clone());
                (!args.is_empty()).then_some(args)
            } else {
                None
            }
        });
        let network_flag = generate_network_flag(rpc_url, network);
        let accounts_file_str = accounts_file.to_string();
        let accounts_file = (!accounts_file_str
            .contains("starknet_accounts/starknet_open_zeppelin_accounts.json"))
        .then_some(accounts_file_str);

        Ok(Self {
            account: account.to_string(),
            accounts_file,
            class_hash: response.class_hash,
            arguments,
            network_flag,
        })
    }
}

impl Message for DeployCommandMessage {
    fn text(&self) -> String {
        let mut command = String::from("sncast");

        if let Some(file) = &self.accounts_file {
            write!(command, " --accounts-file {file}").unwrap();
        }

        write!(command, " --account {}", self.account).unwrap();
        write!(
            command,
            " deploy --class-hash {}",
            self.class_hash.into_hex_string()
        )
        .unwrap();

        if let Some(arguments) = &self.arguments {
            write!(command, " --arguments '{arguments}'").unwrap();
        }

        write!(command, " {}", self.network_flag).unwrap();

        formatdoc!(
            "
            To deploy a contract of this class, replace the placeholders in `--arguments` with your actual values, then run:
            {command}
            "
        )
    }

    fn json(&self) -> Value {
        // This message is only helpful in human mode, we don't need it in JSON mode.
        Value::Null
    }
}

fn generate_constructor_placeholder_args(constructor: AbiConstructor) -> String {
    constructor
        .inputs
        .into_iter()
        .map(|input| {
            let input_type = input
                .r#type
                .split("::")
                .last()
                .expect("Failed to get last part of input type");
            format!("<{} ({})>", input.name, input_type)
        })
        .collect::<Vec<String>>()
        .join(", ")
}
