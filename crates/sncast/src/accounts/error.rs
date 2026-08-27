use thiserror::Error;

use crate::accounts::ChainId;

/// Errors raised while validating and resolving native account data.
#[derive(Debug, Error)]
pub enum AccountsError {
    #[error("{kind} cannot be empty")]
    EmptyIdentifier { kind: &'static str },

    #[error(
        "invalid chain ID `{value}`: expected a Cairo short string of at most 31 ASCII characters or a 0x-prefixed hex value"
    )]
    InvalidChainId { value: String },

    #[error("invalid account type `{account_type}`")]
    InvalidAccountType { account_type: String },

    #[error("account is missing required field `{field}` for {operation}")]
    MissingField {
        field: &'static str,
        operation: &'static str,
    },

    #[error(transparent)]
    AccountsFile(#[from] AccountsFileError),
}

#[derive(Debug, Error)]
pub enum AccountsFileError {
    #[error("failed to serialize accounts file to JSON")]
    Serialize {
        #[source]
        source: serde_json::Error,
    },

    #[error("not valid JSON: {error}")]
    InvalidJson { error: serde_json::Error },

    /// `field_path` is `None` when the invalid value is the whole document.
    #[error(
        "invalid value{}: {error}",
        .field_path.as_ref().map(|path| format!(" at `{path}`")).unwrap_or_default()
    )]
    Schema {
        field_path: Option<String>,
        error: serde_json::Error,
    },

    #[error("schema version {version} is not supported")]
    Version { version: String },

    #[error(
        "network `{network}` is not a valid chain ID: expected a Cairo short string of at most 31 ASCII characters or a 0x-prefixed hex value"
    )]
    InvalidNetworkKey { network: String },

    #[error("networks `{first}` and `{second}` both refer to chain ID `{chain_id}`")]
    DuplicateChainId {
        chain_id: ChainId,
        first: String,
        second: String,
    },

    #[error("invalid account `{account}` on network `{network}`: {message}")]
    InvalidAccountEntry {
        network: String,
        account: String,
        message: String,
    },
}
