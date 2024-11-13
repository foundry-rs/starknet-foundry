use super::transformer::transform;
use serde::{Deserialize, Serialize};
use starknet::core::types::ContractClass;
use starknet_types_core::felt::Felt;

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
    pub fn serialized(
        self,
        class_definition: ContractClass,
        function_selector: &Felt,
    ) -> anyhow::Result<Vec<Felt>> {
        transform(&self.expressions, class_definition, function_selector)
    }
}
