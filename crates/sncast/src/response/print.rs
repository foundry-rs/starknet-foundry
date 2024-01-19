use anyhow::{Error, Result};
use console::style;
use serde_json::Value;
use starknet::core::types::FieldElement;
use std::{collections::HashMap, fmt::Display, str::FromStr};

use serde::{Serialize, Serializer};

use crate::NumbersFormat;

use super::structs::CommandResponse;

pub enum OutputFormat {
    Json,
    Human,
}

impl OutputFormat {
    #[must_use]
    pub fn from_flag(json: bool) -> Self {
        if json {
            OutputFormat::Json
        } else {
            OutputFormat::Human
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum OutputValue {
    String(String),
    Array(Vec<OutputValue>),
}

/// Constrained subset of `serde::json`. No nested maps allowed.
type OutputData = Vec<(String, OutputValue)>;

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

pub fn print_command_result<T: CommandResponse>(
    command: &str,
    result: &mut Result<T>,
    numbers_format: NumbersFormat,
    output_format: &OutputFormat,
) -> Result<()> {
    let mut output: OutputData = vec![];
    output.push((
        String::from("command"),
        OutputValue::String(command.to_string()),
    ));
    output.extend(result_as_output_data(result));
    let formatted_output = output
        .into_iter()
        .map(|(k, v)| (k, apply_numbers_formatting(v, numbers_format)))
        .collect();

    for val in pretty_output(formatted_output, output_format)? {
        match result {
            Ok(_) => println!("{val}"),
            Err(_) => eprintln!("{val}"),
        }
    }
    Ok(())
}

fn pretty_output(output: OutputData, output_format: &OutputFormat) -> Result<Vec<String>> {
    match output_format {
        OutputFormat::Json => {
            let json_output: HashMap<String, OutputValue> = output.into_iter().collect();
            let json_string = serde_json::to_string(&json_output)?;
            Ok(vec![json_string])
        }
        OutputFormat::Human => {
            let mut result = vec![];
            for (key, value) in &output {
                let value = value.to_string();
                result.push(format!("{key}: {value}"));
            }
            Ok(result)
        }
    }
}

fn result_as_output_data<T: CommandResponse>(result: &mut Result<T>) -> OutputData {
    match result {
        Ok(response) => {
            let struct_value =
                serde_json::to_value(response).expect("Failed to serialize CommandResponse");
            struct_value_to_output_data(struct_value)
        }
        Err(message) => {
            vec![(
                String::from("error"),
                OutputValue::String(format!("{message:#}")),
            )]
        }
    }
}

fn struct_value_to_output_data(struct_value: Value) -> OutputData {
    match struct_value {
        Value::Object(obj) => obj
            .into_iter()
            .filter(|(_, v)| !(matches!(v, Value::Null)))
            .map(|(k, v)| (k, value_to_output_value(v)))
            .collect(),
        _ => panic!("Expected an object"),
    }
}

fn value_to_output_value(value: Value) -> OutputValue {
    match value {
        Value::Array(a) => OutputValue::Array(a.into_iter().map(value_to_output_value).collect()),
        Value::String(s) => OutputValue::String(s.to_string()),
        s => panic!("{s:?} cannot be auto-serialized to output"),
    }
}

fn apply_numbers_formatting(value: OutputValue, formatting: NumbersFormat) -> OutputValue {
    match value {
        OutputValue::String(input) => {
            if let Ok(field) = FieldElement::from_str(&input) {
                return match formatting {
                    NumbersFormat::Decimal => OutputValue::String(format!("{field:#}")),
                    NumbersFormat::Hex => OutputValue::String(format!("{field:#x}")),
                    NumbersFormat::Default => OutputValue::String(input),
                };
            }
            OutputValue::String(input)
        }
        OutputValue::Array(arr) => {
            let formatted_arr = arr
                .into_iter()
                .map(|item| apply_numbers_formatting(item, formatting))
                .collect();
            OutputValue::Array(formatted_arr)
        }
    }
}

pub fn print_as_warning(error: &Error) {
    let warning_tag = style("Warning:").color256(11);
    println!("{warning_tag} {error}");
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value};

    use crate::response::print::{
        apply_numbers_formatting, struct_value_to_output_data, OutputData, OutputValue,
    };
    use crate::NumbersFormat;

    #[test]
    fn test_format_json_value_force_decimal() {
        let json_value = OutputValue::Array(vec![OutputValue::String(String::from(
            "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
        ))]);

        let actual = apply_numbers_formatting(json_value, NumbersFormat::Decimal);
        let v = "2087021424722619777119509474943472645767659996348769578120564519014510906823";
        let expected = OutputValue::Array(vec![OutputValue::String(String::from(v))]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_format_json_value_leave_default_decimal() {
        let json_value = OutputValue::Array(vec![OutputValue::String(String::from(
            "2087021424722619777119509474943472645767659996348769578120564519014510906823",
        ))]);

        let actual = apply_numbers_formatting(json_value, NumbersFormat::Default);
        let expected = OutputValue::Array(vec![OutputValue::String(String::from(
            "2087021424722619777119509474943472645767659996348769578120564519014510906823",
        ))]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_format_json_value_leave_default_hex() {
        let json_value = OutputValue::Array(vec![OutputValue::String(String::from(
            "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
        ))]);

        let actual = apply_numbers_formatting(json_value, NumbersFormat::Default);
        let expected = OutputValue::Array(vec![OutputValue::String(String::from(
            "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
        ))]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_format_json_value_force_hex() {
        let json_value = OutputValue::Array(vec![OutputValue::String(String::from(
            "2087021424722619777119509474943472645767659996348769578120564519014510906823",
        ))]);

        let actual = apply_numbers_formatting(json_value, NumbersFormat::Hex);
        let expected = OutputValue::Array(vec![OutputValue::String(String::from(
            "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
        ))]);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_struct_value_to_output_data() {
        let mut json_value = Map::new();
        json_value.insert(
            String::from("K"),
            Value::Array(vec![Value::String(String::from("V"))]),
        );
        json_value.insert(String::from("K2"), Value::Null);

        let actual = struct_value_to_output_data(Value::Object(json_value));
        let json_value_exp: OutputData = vec![(
            String::from("K"),
            OutputValue::Array(vec![OutputValue::String(String::from("V"))]),
        )];
        assert_eq!(actual, json_value_exp);
    }
}
