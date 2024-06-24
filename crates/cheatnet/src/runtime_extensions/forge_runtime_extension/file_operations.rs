use anyhow::{anyhow, Result};
use cairo_felt::{Felt252, ParseFeltError};
use conversions::{
    felt252::{SerializeAsFelt252Vec, TryInferFormat},
    string::TryFromDecStr,
};
use flatten_serde_json::flatten;
use runtime::EnhancedHintError;
use serde_json::{Map, Value};
use std::fs::read_to_string;

pub(super) fn read_txt(path: String) -> Result<Vec<Felt252>, EnhancedHintError> {
    Ok(read_to_string(&path)?
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::trim)
        .map(Felt252::infer_format_and_parse)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| EnhancedHintError::FileParsing { path })?
        .into_iter()
        .flatten()
        .collect())
}

pub(super) fn read_json(path: String) -> Result<Vec<Felt252>, EnhancedHintError> {
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

fn value_into_vec(value: &Value, output: &mut Vec<Felt252>) -> Result<(), ParseFeltError> {
    match value {
        Value::Array(vec) => {
            output.push(vec.len().into());

            for value in vec {
                value_into_vec(value, output)?;
            }

            Ok(())
        }
        Value::Number(num) => {
            output.push(Felt252::try_from_dec_str(&num.to_string())?);

            Ok(())
        }
        Value::String(string) => {
            output.extend(string.as_str().serialize_as_felt252_vec());

            Ok(())
        }
        _ => {
            unreachable!("flatten_serde_json::flatten leaves only numbers string and array of numbers and strings");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::read_json;
    use cairo_felt::Felt252;
    use conversions::felt252::SerializeAsFelt252Vec;
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
        let mut expected_result = vec![
            Felt252::from(1),
            Felt252::from(2),
            Felt252::from(12),
            Felt252::from(43),
        ];
        expected_result.extend("Joh".serialize_as_felt252_vec());

        assert_eq!(result, expected_result);

        let string = r#"
        {
            "ad": "string",
            "test": ["1",2,"3",4]
        }"#;
        let (_temp, file_path) = create_file(string);
        let result = read_json(file_path).unwrap();
        let mut expected_result = "string".serialize_as_felt252_vec();
        expected_result.push(Felt252::from(4));
        expected_result.extend("1".serialize_as_felt252_vec());
        expected_result.push(Felt252::from(2));
        expected_result.extend("3".serialize_as_felt252_vec());
        expected_result.push(Felt252::from(4));

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
