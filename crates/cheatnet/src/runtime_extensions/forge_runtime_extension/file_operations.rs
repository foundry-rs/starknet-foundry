use anyhow::{anyhow, Result};
use cairo_felt::Felt252;
use conversions::{byte_array::ByteArray, felt252::TryInferFormat};
use flatten_serde_json::flatten;
use runtime::EnhancedHintError;
use serde_json::{Map, Value};
use std::fs::read_to_string;

pub(super) fn read_txt(path: String) -> Result<Vec<Felt252>, EnhancedHintError> {
    read_to_string(&path)?
        .lines()
        .filter(|line| !line.is_empty())
        .map(Felt252::infer_format_and_parse)
        .collect::<Result<_, _>>()
        .map_err(|_| EnhancedHintError::FileParsing { path })
}

pub(super) fn read_json(path: String) -> Result<Vec<Felt252>, EnhancedHintError> {
    let content = read_to_string(&path)?;

    let json: Map<String, Value> = serde_json::from_str(&content)
        .map_err(|e| anyhow!("Parse JSON error: {} ", e.to_string()))?;
    let data = flatten(&json);

    let mut result = vec![];
    let mut keys: Vec<&String> = data.keys().collect();

    keys.sort_by_key(|a| a.to_lowercase());

    keys.into_iter()
        .map(|key| value_into_vec(data.get(key).unwrap(), &mut result))
        .collect::<Result<(), _>>()
        .map_err(|_| EnhancedHintError::FileParsing { path })?;

    Ok(result)
}

fn value_into_vec(value: &Value, output: &mut Vec<Felt252>) -> Result<(), ()> {
    match value {
        Value::Array(vec) => {
            for value in vec {
                value_into_vec(value, output)?;
            }

            Ok(())
        }
        Value::Number(num) => {
            output.push(num.as_u64().ok_or(())?.into());

            Ok(())
        }
        Value::String(string) => {
            output.append(&mut ByteArray::from(string.as_str()).serialize_no_magic());

            Ok(())
        }
        _ => {
            unreachable!("flatten_serde_json::flatten leaves only numbers string and array of numbers and strings");
        }
    }
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn test_json_values_sorted_by_keys() {
    //     let string = r#"
    //     {
    //         "name": "Joh",
    //         "age": 43,
    //         "a": {
    //             "b": 1,
    //             "c": 2
    //         },
    //         "ab": 12
    //     }"#;
    //     let result = {
    //         let json: Map<String, Value> = serde_json::from_str(string)
    //             .map_err(|e| anyhow!("Parse JSON error: {} ", e.to_string()))?;
    //         let data = flatten(&json);

    //         let mut keys: Vec<&String> = data.keys().collect();
    //         keys.sort_by_key(|a| a.to_lowercase());

    //         let mut result = vec![];

    //         for key in keys {
    //             value_into_vec(data.get(key).unwrap(), &mut result);
    //         }

    //         Ok(result)
    //     }
    //     .unwrap();
    //     let expected_result = ["1", "2", "12", "43", "\"Joh\""].to_vec();

    //     assert_eq!(result, expected_result);

    //     let string = r#"
    //     {
    //         "ad": "string",
    //         "test": ["1",2,"3",4]
    //     }"#;
    //     let result = {
    //         let json: Map<String, Value> = serde_json::from_str(string)
    //             .map_err(|e| anyhow!("Parse JSON error: {} ", e.to_string()))?;
    //         let data = flatten(&json);

    //         let mut keys: Vec<&String> = data.keys().collect();
    //         keys.sort_by_key(|a| a.to_lowercase());

    //         let mut result = vec![];

    //         for key in keys {
    //             value_into_vec(data.get(key).unwrap(), &mut result);
    //         }

    //         Ok(result)
    //     }
    //     .unwrap();
    //     let expected_result = ["\"string\"", "4", "\"1\"", "2", "\"3\"", "4"];
    //     assert_eq!(result, expected_result);
    // }

    // #[test]
    // fn test_json_values_sorted_by_keys_invalid_data() {
    //     let string = r"
    //     [1,2,'3232']";
    //     let result = {
    //         let json: Map<String, Value> = serde_json::from_str(string)
    //             .map_err(|e| anyhow!("Parse JSON error: {} ", e.to_string()))?;
    //         let data = flatten(&json);

    //         let mut keys: Vec<&String> = data.keys().collect();
    //         keys.sort_by_key(|a| a.to_lowercase());

    //         let mut result = vec![];

    //         for key in keys {
    //             value_into_vec(data.get(key).unwrap(), &mut result);
    //         }

    //         Ok(result)
    //     };
    //     assert!(result.is_err());

    //     let string = r#"
    //     {
    //         "test": 'invalid json format'
    //     }"#;
    //     let result = {
    //         let json: Map<String, Value> = serde_json::from_str(string)
    //             .map_err(|e| anyhow!("Parse JSON error: {} ", e.to_string()))?;
    //         let data = flatten(&json);

    //         let mut keys: Vec<&String> = data.keys().collect();
    //         keys.sort_by_key(|a| a.to_lowercase());

    //         let mut result = vec![];

    //         for key in keys {
    //             value_into_vec(data.get(key).unwrap(), &mut result);
    //         }

    //         Ok(result)
    //     };
    //     assert!(result.is_err());
    // }
}
