// // use super::structs::CommandResponse;
// use crate::NumbersFormat;
// use anyhow::Result;
// // use foundry_ui::OutputFormat;
// // use itertools::Itertools;
// use serde::{Serialize, Serializer};
// use serde_json::Value;
// use starknet_types_core::felt::Felt;
// use std::{
//     // collections::HashMap,
//     fmt::Display,
//     str::FromStr,
// };

// pub trait Format
// where
//     Self: Sized,
// {
//     #[must_use]
//     fn format_with(self, _: NumbersFormat) -> Self;
// }

// #[derive(Clone, PartialEq, Eq, Debug)]
// pub enum OutputValue {
//     String(String),
//     Array(Vec<OutputValue>),
// }

// impl Serialize for OutputValue {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         match &self {
//             OutputValue::String(s) => serializer.serialize_str(s),
//             OutputValue::Array(arr) => arr.serialize::<S>(serializer),
//         }
//     }
// }

// impl Display for OutputValue {
//     fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match &self {
//             OutputValue::String(s) => s.fmt(fmt),
//             OutputValue::Array(arr) => {
//                 let arr_as_string = arr
//                     .iter()
//                     .map(ToString::to_string)
//                     .collect::<Vec<String>>()
//                     .join(", ");
//                 write!(fmt, "[{arr_as_string}]")
//             }
//         }
//     }
// }

// impl From<Value> for OutputValue {
//     fn from(value: Value) -> Self {
//         match value {
//             Value::Array(a) => OutputValue::Array(
//                 a.into_iter()
//                     .map(<OutputValue as From<Value>>::from)
//                     .collect(),
//             ),
//             Value::String(s) => OutputValue::String(s.to_string()),
//             Value::Bool(b) => OutputValue::String(b.to_string()),
//             s => panic!("{s:?} cannot be auto-serialized to output"),
//         }
//     }
// }

// impl Format for OutputValue {
//     fn format_with(self, numbers: NumbersFormat) -> Self {
//         match self {
//             OutputValue::String(input) => {
//                 if let Ok(field) = Felt::from_str(&input) {
//                     return match numbers {
//                         NumbersFormat::Decimal => OutputValue::String(format!("{field:#}")),
//                         NumbersFormat::Hex if input.len() == 66 && input.starts_with("0x0") => {
//                             OutputValue::String(input)
//                         }
//                         NumbersFormat::Hex => OutputValue::String(format!("{field:#x}")),
//                         NumbersFormat::Default => OutputValue::String(input),
//                     };
//                 }
//                 OutputValue::String(input)
//             }
//             OutputValue::Array(arr) => {
//                 let formatted_arr = arr
//                     .into_iter()
//                     .map(|item| item.format_with(numbers))
//                     .collect();
//                 OutputValue::Array(formatted_arr)
//             }
//         }
//     }
// }

// /// Constrained subset of `serde::json`. No nested maps allowed.
// #[derive(Debug, PartialEq, Eq, Serialize)]
// struct OutputData(Vec<(String, OutputValue)>);

// impl Format for OutputData {
//     fn format_with(self, numbers: NumbersFormat) -> Self {
//         Self(
//             self.0
//                 .into_iter()
//                 .map(|(k, v)| (k, v.format_with(numbers)))
//                 .collect(),
//         )
//     }
// }

// // impl<T: CommandResponse> From<&Result<T, anyhow::Error>> for OutputData {
// //     fn from(value: &Result<T>) -> Self {
// //         match value {
// //             Ok(response) => serde_json::to_value(response)
// //                 .expect("Failed to serialize CommandResponse")
// //                 .into(),
// //             Err(message) => Self(vec![(
// //                 String::from("error"),
// //                 OutputValue::String(format!("{message:#}")),
// //             )]),
// //         }
// //     }
// // }

// impl From<Value> for OutputData {
//     fn from(value: Value) -> Self {
//         match value {
//             Value::Object(obj) => {
//                 let pairs = obj
//                     .into_iter()
//                     .filter(|(_, v)| !(matches!(v, Value::Null)))
//                     .map(|(k, v)| (k, v.into()))
//                     .collect();

//                 Self(pairs)
//             }
//             _ => panic!("Expected an object"),
//         }
//     }
// }

// // impl OutputData {
// //     fn to_json(&self, command: &str) -> Result<String> {
// //         let mut mapping: HashMap<_, _> = self.0.clone().into_iter().collect();
// //         mapping.insert(
// //             String::from("command"),
// //             OutputValue::String(command.to_owned()),
// //         );
// //         serde_json::to_string(&mapping).map_err(anyhow::Error::from)
// //     }

// //     fn to_lines(&self, command: &str) -> String {
// //         let fields = self
// //             .0
// //             .iter()
// //             .map(|(key, val)| format!("{key}: {val}"))
// //             .join("\n");

// //         format!("command: {command}\n{fields}")
// //     }

// //     fn to_string_pretty(&self, command: &str, output_format: OutputFormat) -> Result<String> {
// //         match output_format {
// //             OutputFormat::Json => self.to_json(command),
// //             OutputFormat::Human => Ok(self.to_lines(command)),
// //         }
// //     }
// // }

// // pub fn print_command_result<T: CommandResponse>(
// //     command: &str,
// //     result: &Result<T>,
// //     numbers_format: NumbersFormat,
// //     output_format: OutputFormat,
// // ) -> Result<()> {
// //     let output: OutputData = result.into();

// //     let repr = output
// //         .format_with(numbers_format)
// //         .to_string_pretty(command, output_format)?;

// //     match result {
// //         Ok(_) => println!("{repr}"),
// //         Err(_) => eprintln!("{repr}"),
// //     }

// //     Ok(())
// // }

// #[cfg(test)]
// mod tests {
//     use super::{OutputData, OutputValue};
//     use crate::{NumbersFormat, response::print::Format};
//     use serde_json::{Map, Value};

//     #[test]
//     fn test_format_json_value_force_decimal() {
//         let json_value = OutputValue::Array(vec![OutputValue::String(String::from(
//             "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
//         ))]);

//         let actual = json_value.format_with(NumbersFormat::Decimal);
//         let v = "2087021424722619777119509474943472645767659996348769578120564519014510906823";
//         let expected = OutputValue::Array(vec![OutputValue::String(String::from(v))]);
//         assert_eq!(actual, expected);
//     }

//     #[test]
//     fn test_format_json_value_leave_default_decimal() {
//         let json_value = OutputValue::Array(vec![OutputValue::String(String::from(
//             "2087021424722619777119509474943472645767659996348769578120564519014510906823",
//         ))]);

//         let actual = json_value.format_with(NumbersFormat::Default);
//         let expected = OutputValue::Array(vec![OutputValue::String(String::from(
//             "2087021424722619777119509474943472645767659996348769578120564519014510906823",
//         ))]);
//         assert_eq!(actual, expected);
//     }

//     #[test]
//     fn test_format_json_value_leave_default_hex() {
//         let json_value = OutputValue::Array(vec![OutputValue::String(String::from(
//             "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
//         ))]);

//         let actual = json_value.format_with(NumbersFormat::Default);
//         let expected = OutputValue::Array(vec![OutputValue::String(String::from(
//             "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
//         ))]);
//         assert_eq!(actual, expected);
//     }

//     #[test]
//     fn test_format_json_value_force_hex() {
//         let json_value = OutputValue::Array(vec![OutputValue::String(String::from(
//             "2087021424722619777119509474943472645767659996348769578120564519014510906823",
//         ))]);

//         let actual = json_value.format_with(NumbersFormat::Hex);
//         let expected = OutputValue::Array(vec![OutputValue::String(String::from(
//             "0x49d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7",
//         ))]);
//         assert_eq!(actual, expected);
//     }

//     #[test]
//     fn format_address_force_hex() {
//         let json_value = OutputValue::Array(vec![OutputValue::String(String::from(
//             "0x0163a86513df426f4fd7ad989b11062769b03d3fd75fb83fae6c0f416b33a3d5",
//         ))]);

//         let actual = json_value.format_with(NumbersFormat::Hex);
//         let expected = OutputValue::Array(vec![OutputValue::String(String::from(
//             "0x0163a86513df426f4fd7ad989b11062769b03d3fd75fb83fae6c0f416b33a3d5",
//         ))]);

//         assert_eq!(actual, expected);
//     }

//     #[test]
//     fn format_address_force_decimal() {
//         let json_value = OutputValue::Array(vec![OutputValue::String(String::from(
//             "0x0163a86513df426f4fd7ad989b11062769b03d3fd75fb83fae6c0f416b33a3d5",
//         ))]);

//         let actual = json_value.format_with(NumbersFormat::Decimal);
//         let expected = OutputValue::Array(vec![OutputValue::String(String::from(
//             "628392926429977811333168641010360117621605580210734736624161546314682966997",
//         ))]);

//         assert_eq!(actual, expected);
//     }

//     #[test]
//     fn format_address_leave_default() {
//         let json_value = OutputValue::Array(vec![OutputValue::String(String::from(
//             "0x0163a86513df426f4fd7ad989b11062769b03d3fd75fb83fae6c0f416b33a3d5",
//         ))]);

//         let actual = json_value.format_with(NumbersFormat::Default);
//         let expected = OutputValue::Array(vec![OutputValue::String(String::from(
//             "0x0163a86513df426f4fd7ad989b11062769b03d3fd75fb83fae6c0f416b33a3d5",
//         ))]);

//         assert_eq!(actual, expected);
//     }

//     #[test]
//     fn test_struct_value_to_output_data() {
//         let mut json_value = Map::new();
//         json_value.insert(
//             String::from("K"),
//             Value::Array(vec![Value::String(String::from("V"))]),
//         );
//         json_value.insert(String::from("K2"), Value::Null);

//         let actual: OutputData = Value::Object(json_value).into();

//         let expected = OutputData(vec![(
//             String::from("K"),
//             OutputValue::Array(vec![OutputValue::String(String::from("V"))]),
//         )]);

//         assert_eq!(actual, expected);
//     }
// }
