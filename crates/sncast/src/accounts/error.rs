use camino::Utf8PathBuf;
use thiserror::Error;

/// Errors raised while validating and resolving native account data.
#[derive(Debug, Error)]
pub enum AccountsError {
    #[error("account `{account}` was not found for network `{network}`")]
    AccountNotFound { network: String, account: String },

    #[error("account `{account}` already exists on network `{network}`")]
    DuplicateAccount { network: String, account: String },

    #[error("{kind} cannot be empty")]
    EmptyIdentifier { kind: &'static str },

    #[error("accounts file `{path}` does not exist")]
    FileNotFound { path: Utf8PathBuf },

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

    #[error("failed to {operation} the {file_type} `{path}`")]
    Storage {
        operation: StorageOperation,
        file_type: FileType,
        path: Utf8PathBuf,
        #[source]
        source: std::io::Error,
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

#[derive(Clone, Copy, Debug, strum_macros::Display)]
pub enum StorageOperation {
    #[strum(to_string = "create")]
    Create,

    #[strum(to_string = "read")]
    Read,

    #[strum(to_string = "write")]
    Write,

    #[strum(to_string = "inspect")]
    Inspect,

    #[strum(to_string = "replace")]
    Replace,

    #[strum(to_string = "sync")]
    Sync,

    #[strum(to_string = "flush")]
    Flush,

    #[strum(to_string = "set permissions on")]
    SetPermissions,

    #[strum(to_string = "lock")]
    Lock,

    #[strum(to_string = "unlock")]
    Unlock,
}

#[derive(Clone, Copy, Debug, strum_macros::Display)]
pub enum FileType {
    #[strum(to_string = "accounts file")]
    AccountsFile,

    #[strum(to_string = "temporary accounts file")]
    TemporaryAccountsFile,

    #[strum(to_string = "v1 backup of the accounts file")]
    Backup,

    #[strum(to_string = "parent directory of the accounts file")]
    ParentDirectory,

    #[strum(to_string = "lock file")]
    AccountsLockFile,
}
