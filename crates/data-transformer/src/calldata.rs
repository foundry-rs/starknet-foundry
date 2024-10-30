use super::transformer::transform;
use serde::{Deserialize, Serialize};
use starknet::core::types::{ContractClass, Felt};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Calldata {
    expressions: String,
}

impl Calldata {
    #[must_use]
    pub fn new(expressions: String) -> Self {
        Self { expressions }
    }
}

impl Calldata {
    /// Serialize the calldata.
    /// If it's given as a list of `Felt`s, return it immediately.
    /// Otherwise, try to interpret is as a comma-separated sequence of Cairo expressions.
    pub fn serialized(
        self,
        class_definition: ContractClass,
        function_selector: &Felt,
    ) -> anyhow::Result<Vec<Felt>> {
        transform(&self.expressions, class_definition, function_selector)
    }
}
