use starknet_types_core::felt::Felt;

use crate::runtime_extensions::call_to_blockifier_runtime_extension::panic_parser::{
    error_contains_constructor_selector, try_extract_panic_data,
};

pub enum Panic {
    InConstructor(Vec<Felt>),
    InEntrypoint(Vec<Felt>),
}

impl Panic {
    pub fn try_from_error(error: &str) -> Option<Self> {
        let panic_data = try_extract_panic_data(error);

        if let Some(data) = panic_data {
            if error_contains_constructor_selector(error) {
                Some(Panic::InConstructor(data))
            } else {
                Some(Panic::InEntrypoint(data))
            }
        } else {
            None
        }
    }
}
