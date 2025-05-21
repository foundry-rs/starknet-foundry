use foundry_ui::formats::{NumbersFormat, OutputFormat};
use std::{collections::HashMap, fmt::Display, str::FromStr};

use anyhow::Result;
use serde::{Serialize, Serializer};
use serde_json::Value;
use starknet_types_core::felt::Felt;

use super::structs::CommandResponse;

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

impl<T: CommandResponse> From<&Result<T, anyhow::Error>> for OutputData {
    fn from(value: &Result<T>) -> Self {
        match value {
            Ok(response) => serde_json::to_value(response)
                .expect("Failed to serialize CommandResponse")
                .into(),
            Err(message) => Self(vec![(
                String::from("error"),
                OutputValue::String(format!("{message:#}")),
            )]),
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

/// Constrained subset of `serde::json`. No nested maps allowed.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct OutputData(Vec<(String, OutputValue)>);

impl Format for OutputData {
    fn format_with(self, numbers: NumbersFormat) -> Self {
        Self(
            self.0
                .into_iter()
                .map(|(k, v)| (k, v.format_with(numbers)))
                .collect(),
        )
    }
}

impl From<Value> for OutputData {
    fn from(value: Value) -> Self {
        match value {
            Value::Object(obj) => {
                let pairs = obj
                    .into_iter()
                    .filter(|(_, v)| !(matches!(v, Value::Null)))
                    .map(|(k, v)| (k, v.into()))
                    .collect();

                Self(pairs)
            }
            _ => panic!("Expected an object"),
        }
    }
}

impl<T: CommandResponse + Serialize> From<&T> for OutputData {
    fn from(value: &T) -> Self {
        serde_json::to_value(value)
            .expect("Failed to serialize CommandResponse")
            .into()
    }
}

impl OutputData {
    fn to_json(&self) -> Result<String> {
        let mapping: HashMap<_, _> = self.0.clone().into_iter().collect();
        serde_json::to_string(&mapping).map_err(anyhow::Error::from)
    }

    fn to_lines(&self) -> String {
        let mut rest = self.0.clone();
        let command_val = rest
            .iter()
            .position(|(k, _)| k == "command")
            .map(|idx| rest.remove(idx).1);

        let fields = rest
            .iter()
            .map(|(key, val)| format!("{key}: {val}"))
            .collect::<Vec<_>>();

        match command_val {
            Some(command) => format!("command: {command}\n{}", fields.join("\n")),
            None => fields.join("\n"),
        }
    }

    pub fn to_string_pretty(&self, output_format: OutputFormat) -> Result<String> {
        match output_format {
            OutputFormat::Json => self.to_json(),
            OutputFormat::Human => Ok(self.to_lines()),
        }
    }
}
