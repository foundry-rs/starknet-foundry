use super::transformer::transform;
use serde::{Deserialize, Serialize};
use starknet::core::types::{ContractClass, Felt};

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
                if arg
                    .chars()
                    .all(|c| c.is_ascii_hexdigit() || c == 'x' || c == 'X')
                {
                    Felt::from_dec_str(arg)
                        .or_else(|_| Felt::from_hex(arg))
                        .ok()
                } else {
                    None
                }
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
    /// Serialize the calldata.
    /// If it's given as a list of `Felt`s, return it immediately.
    /// Otherwise, try to interpret is as a comma-separated sequence of Cairo expressions.
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
