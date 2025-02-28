use anyhow::{anyhow, Result};
use conversions::felt::TryInferFormat;
use conversions::{
    byte_array::ByteArray, serde::serialize::SerializeToFeltVec, string::TryFromDecStr,
};
use flatten_serde_json::flatten;
use runtime::EnhancedHintError;
use serde_json::{Map, Value};
use starknet_types_core::felt::Felt;
use starknet_types_core::felt::FromStrError;
use std::fs::read_to_string;

pub(super) fn read_txt(path: String) -> Result<Vec<Felt>, EnhancedHintError> {
    Ok(read_to_string(&path)?
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::trim)
        .map(Felt::infer_format_and_parse)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| EnhancedHintError::FileParsing { path })?
        .into_iter()
        .flatten()
        .collect())
}

pub(super) fn read_json(path: String) -> Result<Vec<Felt>, EnhancedHintError> {
    let content = read_to_string(&path)?;

    let json: Map<String, Value> = serde_json::from_str(&content)
        .map_err(|e| anyhow!("Parse JSON error: {} , in file {path}", e.to_string()))?;
    let data = flatten(&json);

    let mut result = vec![];
    let mut keys: Vec<_> = data.keys().collect();

    keys.sort_by_key(|a| a.to_lowercase());

    keys.into_iter()
        .try_for_each(|key| value_into_vec(data.get(key).unwrap(), &mut result))
        .map_err(|_| EnhancedHintError::FileParsing { path })?;

    Ok(result)
}

fn value_into_vec(value: &Value, output: &mut Vec<Felt>) -> Result<(), FromStrError> {
    match value {
        Value::Array(vec) => {
            output.push(vec.len().into());

            for value in vec {
                value_into_vec(value, output)?;
            }

            Ok(())
        }
        Value::Number(num) => {
            output.push(Felt::try_from_dec_str(&num.to_string())?);

            Ok(())
        }
        Value::String(string) => {
            output.extend(ByteArray::from(string.as_str()).serialize_to_vec());

            Ok(())
        }
        _ => {
            unreachable!(
                "flatten_serde_json::flatten leaves only numbers string and array of numbers and strings"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::read_json;
    use conversions::{byte_array::ByteArray, serde::serialize::SerializeToFeltVec};
    use starknet_types_core::felt::Felt;
    use std::fs;
    use tempfile::TempDir;

    fn create_file(content: impl AsRef<[u8]>) -> (TempDir, String) {
        let temp = TempDir::new().unwrap();
        let file = format!("{}/file.json", temp.path().to_string_lossy());

        fs::write(&file, content).unwrap();

        (temp, file)
    }

    #[test]
    fn test_json_values_sorted_by_keys() {
        let string = r#"
        {
            "name": "Joh",
            "age": 43,
            "a": {
                "b": 1,
                "c": 2
            },
            "ab": 12
        }"#;
        let (_temp, file_path) = create_file(string);
        let result = read_json(file_path).unwrap();
        let mut expected_result =
            vec![Felt::from(1), Felt::from(2), Felt::from(12), Felt::from(43)];
        expected_result.extend(ByteArray::from("Joh").serialize_to_vec());

        assert_eq!(result, expected_result);

        let string = r#"
        {
            "ad": "string",
            "test": ["1",2,"3",4]
        }"#;
        let (_temp, file_path) = create_file(string);
        let result = read_json(file_path).unwrap();
        let mut expected_result = ByteArray::from("string").serialize_to_vec();
        expected_result.push(Felt::from(4));
        expected_result.extend(ByteArray::from("1").serialize_to_vec());
        expected_result.push(Felt::from(2));
        expected_result.extend(ByteArray::from("3").serialize_to_vec());
        expected_result.push(Felt::from(4));

        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_json_values_sorted_by_keys_invalid_data() {
        let string = r"
        [1,2,'3232']";
        let (_temp, file_path) = create_file(string);
        let result = read_json(file_path);
        assert!(result.is_err());

        let string = r#"
        {
            "test": 'invalid json format'
        }"#;
        let (_temp, file_path) = create_file(string);
        let result = read_json(file_path);
        assert!(result.is_err());
    }
}
