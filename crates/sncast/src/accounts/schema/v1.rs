use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use starknet_rust::signers::DerivationPath;
use starknet_types_core::felt::Felt;

use crate::accounts::AccountType;

/// The legacy, unversioned accounts file. Its permissive signer model is intentionally
/// isolated in this module and must not be reused for new writes.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct AccountsFile(pub BTreeMap<String, BTreeMap<String, Account>>);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Account {
    pub public_key: Felt,
    pub address: Option<Felt>,
    pub salt: Option<Felt>,
    pub deployed: Option<bool>,
    pub class_hash: Option<Felt>,
    pub legacy: Option<bool>,

    #[serde(default, rename = "type")]
    pub account_type: Option<AccountType>,

    #[serde(flatten)]
    pub signer: Signer,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Signer {
    PrivateKey { private_key: Felt },
    Ledger { ledger_path: DerivationPath },
}
