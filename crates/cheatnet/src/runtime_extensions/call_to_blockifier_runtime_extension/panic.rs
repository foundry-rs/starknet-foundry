use starknet_types_core::felt::Felt;

use crate::runtime_extensions::call_to_blockifier_runtime_extension::panic_parser::{
    error_contains_constructor_selector, try_extract_panic_data,
};

/// Represents a panic raised during contract execution.
///
/// - `Constructor`: panic raised inside a contract constructor.
/// - `Entrypoint`: panic raised inside a normal entrypoint call.
pub enum Panic {
    Constructor(Vec<Felt>),
    Entrypoint(Vec<Felt>),
}

impl Panic {
    /// Attempts to classify the panic from a raw Starknet error string.
    ///
    /// Returns:
    /// - `Some(Panic)` if panic data can be extracted,
    /// - `None` if the error does not contain panic data.
    #[must_use]
    pub fn try_from_error(error: &str) -> Option<Self> {
        let panic_data = try_extract_panic_data(error)?;

        if error_contains_constructor_selector(error) {
            Some(Panic::Constructor(panic_data))
        } else {
            Some(Panic::Entrypoint(panic_data))
        }
    }
}
