use camino::Utf8PathBuf;
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

    #[error("failed to {operation} `{path}`: {source}")]
    Storage {
        operation: &'static str,
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("accounts file `{path}` does not exist")]
    FileNotFound { path: Utf8PathBuf },

    #[error("refusing to access accounts file through symlink `{path}`")]
    Symlink { path: Utf8PathBuf },

    #[error("account `{account}` already exists on network `{network}`")]
    DuplicateAccount { network: String, account: String },

    #[error("account `{account}` was not found on network `{network}`")]
    AccountNotFound { network: String, account: String },

    #[error("invalid account: {message}")]
    InvalidAccount { message: String },
}
