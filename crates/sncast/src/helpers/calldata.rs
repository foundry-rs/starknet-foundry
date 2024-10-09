use serde::{Deserialize, Serialize};
use starknet::core::types::{ContractClass, Felt};

use super::data_transformer::transformer::transform;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Calldata {
    Serialized(Vec<Felt>),
    Expressions(String),
}

impl From<Vec<String>> for Calldata {
    fn from(calldata: Vec<String>) -> Self {
        let maybe_serialized = calldata
            .iter()
            .map(|arg| {
                Felt::from_dec_str(arg)
                    .or_else(|_| Felt::from_hex(arg))
                    .ok()
            })
            .collect::<Option<Vec<_>>>();

        if let Some(serialized) = maybe_serialized {
            Self::Serialized(serialized)
        } else {
            Self::Expressions(calldata.join(" "))
        }
    }
}

impl Calldata {
    /// Serialize Calldata if it's given as a list of Cairo-like expressions (Strings),
    /// return it as already serialized otherwise
    pub fn serialized(
        self,
        class_definition: ContractClass,
        function_selector: &Felt,
    ) -> anyhow::Result<Vec<Felt>> {
        match self {
            Calldata::Serialized(serialized) => Ok(serialized),
            Calldata::Expressions(ref expressions) => {
                transform(expressions, class_definition, function_selector)
            }
        }
    }
}
