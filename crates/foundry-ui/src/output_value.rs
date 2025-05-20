use std::{fmt::Display, str::FromStr};

use serde::{Serialize, Serializer};
use serde_json::Value;
use starknet_types_core::felt::Felt;

use crate::formats::NumbersFormat;

pub trait Format
where
    Self: Sized,
{
    #[must_use]
    fn format_with(self, _: NumbersFormat) -> Self;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum OutputValue {
    String(String),
    Array(Vec<OutputValue>),
}

impl Serialize for OutputValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match &self {
            OutputValue::String(s) => serializer.serialize_str(s),
            OutputValue::Array(arr) => arr.serialize::<S>(serializer),
        }
    }
}

impl Display for OutputValue {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            OutputValue::String(s) => s.fmt(fmt),
            OutputValue::Array(arr) => {
                let arr_as_string = arr
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(fmt, "[{arr_as_string}]")
            }
        }
    }
}

impl From<Value> for OutputValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Array(a) => OutputValue::Array(
                a.into_iter()
                    .map(<OutputValue as From<Value>>::from)
                    .collect(),
            ),
            Value::String(s) => OutputValue::String(s.to_string()),
            Value::Bool(b) => OutputValue::String(b.to_string()),
            s => panic!("{s:?} cannot be auto-serialized to output"),
        }
    }
}

impl Format for OutputValue {
    fn format_with(self, numbers: NumbersFormat) -> Self {
        match self {
            OutputValue::String(input) => {
                if let Ok(field) = Felt::from_str(&input) {
                    return match numbers {
                        NumbersFormat::Decimal => OutputValue::String(format!("{field:#}")),
                        NumbersFormat::Hex if input.len() == 66 && input.starts_with("0x0") => {
                            OutputValue::String(input)
                        }
                        NumbersFormat::Hex => OutputValue::String(format!("{field:#x}")),
                        NumbersFormat::Default => OutputValue::String(input),
                    };
                }
                OutputValue::String(input)
            }
            OutputValue::Array(arr) => {
                let formatted_arr = arr
                    .into_iter()
                    .map(|item| item.format_with(numbers))
                    .collect();
                OutputValue::Array(formatted_arr)
            }
        }
    }
}
