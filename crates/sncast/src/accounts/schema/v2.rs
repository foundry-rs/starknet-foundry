use std::collections::BTreeMap;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use starknet_types_core::felt::Felt;

use crate::accounts::AccountType;

pub const VERSION: u32 = 2;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AccountsFile {
    pub version: u32,
    pub accounts: BTreeMap<String, BTreeMap<String, Account>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Account {
    pub public_key: Felt,

    pub address: Felt,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<Felt>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployed: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_hash: Option<Felt>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub legacy: Option<bool>,

    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub account_type: Option<AccountType>,

    pub signer: Signer,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Signer {
    PrivateKey {
        private_key: Felt,
    },
    Keystore {
        path: Utf8PathBuf,

        #[serde(skip_serializing_if = "Option::is_none")]
        password_env: Option<String>,
    },
    Ledger {
        derivation_path: String,
    },
}
