use super::explorer_link::OutputLink;
use crate::helpers::block_explorer::LinkProvider;
use crate::response::cast_message::SncastCommandMessage;
use crate::response::dry_run::DryRunResponse;
use anyhow::Error;
use camino::Utf8PathBuf;
use conversions::string::IntoHexStr;
use conversions::{IntoConv, padded_felt::PaddedFelt, serde::serialize::CairoSerialize};
use foundry_ui::{Message, styling};
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

impl SncastCommandMessage for DeclareTransactionResponse {
    fn text(&self) -> String {
        styling::OutputBuilder::new()
            .success_message("Declaration completed")
            .blank_line()
            .field("Class Hash", &self.class_hash.into_hex_string())
            .field("Transaction Hash", &self.transaction_hash.into_hex_string())
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
    #[serde(untagged)]
    DryRun(DryRunResponse),
}

impl DeclareResponse {
    #[must_use]
    pub fn class_hash(&self) -> Felt {
        match self {
            DeclareResponse::AlreadyDeclared(response) => response.class_hash.into_(),
            DeclareResponse::Success(response) => response.class_hash.into_(),
            DeclareResponse::DryRun(_) => unreachable!(
                "Dry run response should not be used to get class hash, as it does not contain one"
            ),
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
    constructor_args: Option<ConstructorArgs>,
    network_flag: String,
}

impl DeployCommandMessage {
    pub fn new(
        abi: &[AbiEntry],
        abi_in_declared_class: bool,
        response: &DeclareTransactionResponse,
        account: &str,
        accounts_file: &Utf8PathBuf,
        network_flag: String,
    ) -> Result<Self, Error> {
        let accounts_file_str = accounts_file.to_string();
        let accounts_file = (!accounts_file_str
            .contains("starknet_accounts/starknet_open_zeppelin_accounts.json"))
        .then_some(accounts_file_str);

        Ok(Self {
            account: account.to_string(),
            accounts_file,
            class_hash: response.class_hash,
            constructor_args: ConstructorArgs::from_abi(abi, abi_in_declared_class),
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

        if let Some(constructor_args) = &self.constructor_args {
            write!(command, " {}", constructor_args.text()).unwrap();
        }

        write!(command, " {}", self.network_flag).unwrap();

        let header = match &self.constructor_args {
            Some(arguments_flag) => format!(
                "To deploy a contract of this class, replace the placeholders in `{}` with your actual values, then run:",
                arguments_flag.flag_name()
            ),
            None => "To deploy a contract of this class, run:".to_string(),
        };

        let hint = self
            .constructor_args
            .as_ref()
            .and_then(ConstructorArgs::hint)
            .map(|hint| format!("\n\nHint: {hint}"))
            .unwrap_or_default();

        formatdoc!(
            "
            {header}
            {command}
            {hint}
            "
        )
    }

    fn json(&self) -> Value {
        // TODO(#3960) JSON output support
        // This message is only helpful in human mode, we don't need it in JSON mode.
        Value::Null
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
enum ConstructorArgs {
    CairoLike(String),
    RawCalldata(String),
}

impl ConstructorArgs {
    fn from_abi(abi: &[AbiEntry], abi_in_declared_class: bool) -> Option<Self> {
        abi.iter().find_map(|entry| {
            let AbiEntry::Constructor(constructor) = entry else {
                return None;
            };
            if constructor.inputs.is_empty() {
                return None;
            }

            let cairo_args = generate_constructor_placeholder_arguments(constructor);

            Some(if abi_in_declared_class {
                Self::CairoLike(cairo_args)
            } else {
                Self::RawCalldata(cairo_args)
            })
        })
    }

    fn flag_name(&self) -> &'static str {
        match self {
            Self::CairoLike(_) => "--arguments",
            Self::RawCalldata(_) => "--constructor-calldata",
        }
    }

    fn text(&self) -> String {
        match self {
            Self::CairoLike(cairo_args) => format!("--arguments '{cairo_args}'"),
            Self::RawCalldata(_) => {
                "--constructor-calldata <serialized-constructor-args>".to_string()
            }
        }
    }

    fn hint(&self) -> Option<String> {
        match self {
            Self::CairoLike(_) => None,
            Self::RawCalldata(cairo_args) => Some(format!(
                "Constructor arguments must be pre-serialized. Use `sncast utils serialize --abi-file <path-to-abi> --function constructor --arguments '{}'` and pass the returned felts to `--constructor-calldata`.",
                cairo_args
            )),
        }
    }
}

fn generate_constructor_placeholder_arguments(constructor: &AbiConstructor) -> String {
    constructor
        .inputs
        .iter()
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

fn generate_accounts_file_flag(accounts_file: Option<&String>) -> Option<String> {
    accounts_file
        .as_ref()
        .map(|file| format!("--accounts-file {file}"))
}
