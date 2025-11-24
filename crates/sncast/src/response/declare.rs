use super::{command::CommandResponse, explorer_link::OutputLink};
use crate::helpers::block_explorer::LinkProvider;
use crate::response::cast_message::{SncastCommandMessage, SncastMessage};
use anyhow::Error;
use camino::Utf8PathBuf;
use conversions::string::IntoHexStr;
use conversions::{IntoConv, padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::Message;
use foundry_ui::styling;
use indoc::formatdoc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use starknet_rust::core::types::contract::{AbiConstructor, AbiEntry};
use starknet_types_core::felt::Felt;
use std::fmt::Write;

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct DeclareTransactionResponse {
    pub class_hash: PaddedFelt,
    pub transaction_hash: PaddedFelt,
}

impl CommandResponse for DeclareTransactionResponse {}

impl SncastCommandMessage for SncastMessage<DeclareTransactionResponse> {
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
}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
pub struct AlreadyDeclaredResponse {
    pub class_hash: PaddedFelt,
}

#[derive(Clone, Serialize, Deserialize, CairoSerialize, Debug, PartialEq)]
#[serde(tag = "status")]
pub enum DeclareResponse {
    AlreadyDeclared(AlreadyDeclaredResponse),
    #[serde(untagged)]
    Success(DeclareTransactionResponse),
}

impl DeclareResponse {
    #[must_use]
    pub fn class_hash(&self) -> Felt {
        match self {
            DeclareResponse::AlreadyDeclared(response) => response.class_hash.into_(),
            DeclareResponse::Success(response) => response.class_hash.into_(),
        }
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
    arguments_flag: Option<String>,
    network_flag: String,
}

impl DeployCommandMessage {
    pub fn new(
        abi: &[AbiEntry],
        response: &DeclareTransactionResponse,
        account: &str,
        accounts_file: &Utf8PathBuf,
        network_flag: String,
    ) -> Result<Self, Error> {
        let arguments_flag: Option<String> = generate_arguments_flag(abi);
        let accounts_file_str = accounts_file.to_string();
        let accounts_file = (!accounts_file_str
            .contains("starknet_accounts/starknet_open_zeppelin_accounts.json"))
        .then_some(accounts_file_str);

        Ok(Self {
            account: account.to_string(),
            accounts_file,
            class_hash: response.class_hash,
            arguments_flag,
            network_flag,
        })
    }
}

impl Message for DeployCommandMessage {
    fn text(&self) -> String {
        let mut command = String::from("sncast");

        let accounts_file_flag = generate_accounts_file_flag(self.accounts_file.as_ref());
        if let Some(flag) = accounts_file_flag {
            write!(command, " {flag}").unwrap();
        }

        let account_flag = format!("--account {}", self.account);
        write!(command, " {account_flag}").unwrap();

        write!(command, " deploy").unwrap();

        write!(
            command,
            " --class-hash {}",
            self.class_hash.into_hex_string()
        )
        .unwrap();

        if let Some(arguments) = &self.arguments_flag {
            write!(command, " {arguments}").unwrap();
        }

        write!(command, " {}", self.network_flag).unwrap();

        let header = if self.arguments_flag.is_some() {
            "To deploy a contract of this class, replace the placeholders in `--arguments` with your actual values, then run:"
        } else {
            "To deploy a contract of this class, run:"
        };

        formatdoc!(
            "
            {header}
            {command}
            "
        )
    }

    fn json(&self) -> Value {
        // This message is only helpful in human mode, we don't need it in JSON mode.
        Value::Null
    }
}

fn generate_constructor_placeholder_arguments(constructor: AbiConstructor) -> String {
    constructor
        .inputs
        .into_iter()
        .map(|input| {
            let input_type = input
                .r#type
                .split("::")
                .last()
                .expect("Failed to get last part of input type");
            format!("<{}: {}>", input.name, input_type)
        })
        .collect::<Vec<String>>()
        .join(", ")
}

fn generate_arguments_flag(abi: &[AbiEntry]) -> Option<String> {
    let arguments = abi.iter().find_map(|entry| {
        if let AbiEntry::Constructor(constructor) = entry {
            let arguments = generate_constructor_placeholder_arguments(constructor.clone());
            (!arguments.is_empty()).then_some(arguments)
        } else {
            None
        }
    });

    arguments.map(|arguments| format!("--arguments '{arguments}'"))
}

fn generate_accounts_file_flag(accounts_file: Option<&String>) -> Option<String> {
    accounts_file
        .as_ref()
        .map(|file| format!("--accounts-file {file}"))
}
