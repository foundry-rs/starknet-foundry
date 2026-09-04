use thiserror::Error;

/// Errors raised while validating and resolving native account data.
#[derive(Debug, Error)]
pub enum AccountsError {
    #[error("{kind} cannot be empty")]
    EmptyIdentifier { kind: &'static str },

    #[error("invalid account type `{account_type}`")]
    InvalidAccountType { account_type: String },

    #[error("account is missing required field `{field}` for {operation}")]
    MissingField {
        field: &'static str,
        operation: &'static str,
    },

    #[error(transparent)]
    AccountsFile(#[from] AccountsFileError),

    #[error("invalid account `{account}` on network `{network}`: {message}")]
    InvalidAccountEntry {
        network: String,
        account: String,
        message: String,
    },
}

#[derive(Debug, Error)]
pub enum AccountsFileError {
    #[error("failed to serialize accounts file to JSON")]
    Serialize {
        #[source]
        source: serde_json::Error,
    },

    #[error("failed to deserialize accounts file from JSON")]
    Deserialize {
        #[source]
        source: serde_json::Error,
    },

    #[error("invalid schema of field {field_path} in the accounts file")]
    Schema {
        field_path: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("accounts file schema version {version} is not supported")]
    Version { version: String },
}
