use crate::response::cast_message::SncastCommandMessage;
use crate::{AccountData, AccountType, SignerType};
use camino::Utf8PathBuf;
use conversions::string::IntoHexStr;
use foundry_ui::styling::OutputBuilder;
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;

#[derive(Serialize, Clone, Debug)]
pub struct AccountListResponse {
    pub accounts_file: Utf8PathBuf,
    /// Accounts grouped by network, each group sorted by name.
    /// Serialized as a flat list.
    #[serde(serialize_with = "serialize_flat")]
    pub accounts: BTreeMap<String, Vec<AccountDataRepresentation>>,
    #[serde(skip)]
    pub display_private_keys: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct AccountDataRepresentation {
    pub network: String,
    pub name: String,
    pub public_key: String,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub signer_type: Option<SignerType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy: Option<bool>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub account_type: Option<AccountType>,
}

impl AccountDataRepresentation {
    #[must_use]
    pub fn new(
        account: AccountData,
        network: String,
        name: String,
        display_private_key: bool,
    ) -> Self {
        let AccountData {
            public_key,
            address,
            salt,
            deployed,
            class_hash,
            legacy,
            account_type,
            signer_type,
        } = account;

        let signer_type = match signer_type {
            SignerType::Local { .. } if !display_private_key => None,
            signer_type => Some(signer_type),
        };

        Self {
            network,
            name,
            public_key: public_key.into_hex_string(),
            signer_type,
            address: address.map(IntoHexStr::into_hex_string),
            salt: salt.map(IntoHexStr::into_hex_string),
            deployed,
            class_hash: class_hash.map(IntoHexStr::into_hex_string),
            legacy,
            account_type,
        }
    }

    fn append_fields(&self, builder: OutputBuilder) -> OutputBuilder {
        builder
            .if_some(self.address.as_ref(), |b, address| {
                b.field("Address", address)
            })
            .if_some(self.account_type.as_ref(), |b, account_type| {
                b.field("Type", &account_type.to_string())
            })
            .if_some(self.deployed.as_ref(), |b, deployed| {
                b.field("Deployed", &deployed.to_string())
            })
            .field("Public Key", &self.public_key)
            .if_some(self.class_hash.as_ref(), |b, class_hash| {
                b.field("Class Hash", class_hash)
            })
            .if_some(self.salt.as_ref(), |b, salt| b.field("Salt", salt))
            .if_some(self.legacy.as_ref(), |b, legacy| {
                b.field("Legacy", &legacy.to_string())
            })
            .if_some(
                self.signer_type.as_ref(),
                |b, signer_type| match signer_type {
                    SignerType::Local { private_key } => {
                        b.field("Private Key", &private_key.into_hex_string())
                    }
                    SignerType::Ledger { ledger_path } => {
                        b.field("Ledger Path", &ledger_path.derivation_string())
                    }
                },
            )
    }
}

impl SncastCommandMessage for AccountListResponse {
    fn text(&self) -> String {
        if self.accounts.is_empty() {
            return OutputBuilder::new()
                .text_field(&format!("No accounts found in {}", self.accounts_file))
                .blank_line()
                .text_field("To create one, run `sncast account create` or `sncast account import`")
                .build();
        }

        let mut builder = OutputBuilder::new().field("Accounts File", self.accounts_file.as_str());

        for (network, accounts) in &self.accounts {
            builder = builder.blank_line().heading(network);

            for (index, account) in accounts.iter().enumerate() {
                if index > 0 {
                    builder = builder.blank_line();
                }

                builder = builder.indented(2, |b| {
                    b.heading(&account.name)
                        .indented(2, |b| account.append_fields(b))
                });
            }
        }

        if !self.display_private_keys {
            builder = builder
                .blank_line()
                .text_field("To show private keys too, run with --display-private-keys or -p");
        }

        builder.build()
    }
}

fn serialize_flat<S>(
    accounts: &BTreeMap<String, Vec<AccountDataRepresentation>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.collect_seq(accounts.values().flatten())
}
