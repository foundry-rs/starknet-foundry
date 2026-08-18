use thiserror::Error;

/// Errors raised while validating and resolving native account data.
#[derive(Debug, Error)]
pub enum AccountsError {
    #[error("{kind} cannot be empty")]
    EmptyIdentifier { kind: &'static str },

    #[error("account is missing required field `{field}` for {operation}")]
    MissingField {
        field: &'static str,
        operation: &'static str,
    },

    #[error("invalid accounts file schema at `{path}`: {message}")]
    Schema { path: String, message: String },

    #[error("accounts file schema version {version} is not supported")]
    UnsupportedVersion { version: String },

    #[error("failed to migrate account `{account}` on network `{network}`: {message}")]
    Migration {
        network: String,
        account: String,
        message: String,
    },
}
