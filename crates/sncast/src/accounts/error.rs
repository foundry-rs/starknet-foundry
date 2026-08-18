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
}
