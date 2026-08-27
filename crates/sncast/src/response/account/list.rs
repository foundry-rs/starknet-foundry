use crate::accounts::schema::v2;
use crate::accounts::{AccountName, AccountRecord, AccountType, ChainId};
use crate::response::cast_message::SncastCommandMessage;
use crate::signers::SignerSpec;
use camino::Utf8PathBuf;
use conversions::string::IntoHexStr;
use foundry_ui::styling::OutputBuilder;
use itertools::Itertools;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AccountListResponse {
    pub accounts_file: Utf8PathBuf,
    /// Sorted by `(chain_id, name)`.
    pub accounts: Vec<AccountDataRepresentation>,
    #[serde(skip)]
    pub display_private_keys: bool,
}

#[derive(Debug, Serialize)]
pub struct AccountDataRepresentation {
    pub name: String,
    pub chain_id: String,
    pub public_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signer: Option<v2::Signer>,
    pub address: String,
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
        account: &AccountRecord,
        chain_id: &ChainId,
        name: &AccountName,
        display_private_key: bool,
    ) -> Self {
        let signer = match &account.signer {
            SignerSpec::PrivateKey(_) if !display_private_key => None,
            SignerSpec::PrivateKey(spec) => Some(v2::Signer::PrivateKey {
                private_key: spec.private_key(),
            }),
            SignerSpec::Keystore(spec) => Some(v2::Signer::Keystore {
                path: spec.path().to_owned(),
                password_env: spec.password_env().map(ToOwned::to_owned),
            }),
            SignerSpec::Ledger(spec) => Some(v2::Signer::Ledger {
                derivation_path: spec.derivation_path().derivation_string(),
            }),
        };

        Self {
            name: name.to_string(),
            chain_id: chain_id.to_string(),
            public_key: account.public_key.into_hex_string(),
            signer,
            address: account.address.into_hex_string(),
            salt: account.salt.map(IntoHexStr::into_hex_string),
            deployed: account.deployed,
            class_hash: account.class_hash.map(IntoHexStr::into_hex_string),
            legacy: account.legacy,
            account_type: account.account_type,
        }
    }

    fn append_fields(&self, builder: OutputBuilder) -> OutputBuilder {
        builder
            .field("Address", &self.address)
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
            .if_some(self.signer.as_ref(), |b, signer| match signer {
                v2::Signer::PrivateKey { private_key } => {
                    b.field("Private Key", &private_key.into_hex_string())
                }
                v2::Signer::Keystore { path, password_env } => b
                    .field("Keystore", path.as_str())
                    .if_some(password_env.as_ref(), |b, password_env| {
                        b.field("Password Env", password_env)
                    }),
                v2::Signer::Ledger { derivation_path } => b.field("Ledger Path", derivation_path),
            })
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

        for (chain_id, accounts) in &self.accounts.iter().chunk_by(|account| &account.chain_id) {
            builder = builder.blank_line().heading(chain_id);

            for (index, account) in accounts.enumerate() {
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
            builder = builder.blank_line().text_field(
                "To include private keys in the output, run with --display-private-keys or -p",
            );
        }

        builder.build()
    }
}
