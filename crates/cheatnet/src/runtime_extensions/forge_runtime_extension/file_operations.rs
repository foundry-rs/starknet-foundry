use anyhow::{anyhow, Result};
use cairo_felt::Felt252;
use conversions::{felt252::TryInferFormat, string::ParseFeltError};
use flatten_serde_json::flatten;
use runtime::EnhancedHintError;
use serde_json::{Map, Value};
use std::fs::read_to_string;

pub(super) fn read_txt(path: String) -> Result<Vec<Felt252>, EnhancedHintError> {
    let content = read_to_string(&path)?;

    content
        .trim()
        .lines()
        .filter(|line| !line.is_empty())
        .map(parse)
        .collect::<Result<_, _>>()
        .map_err(|_| EnhancedHintError::FileParsing { path })
}

pub(super) fn read_json(path: String) -> Result<Vec<Felt252>, EnhancedHintError> {
    let content = read_to_string(&path)?;
    let split_content = json_values_sorted_by_keys(&content)
        .map_err(|e| anyhow!("{}, in file {}", e.to_string(), path))?;

    split_content
        .into_iter()
        .map(|str| parse(&str))
        .collect::<Result<_, _>>()
        .map_err(|_| EnhancedHintError::FileParsing { path })
}

fn json_values_sorted_by_keys(content: &str) -> Result<Vec<String>, EnhancedHintError> {
    let json: Map<String, Value> = serde_json::from_str(content)
        .map_err(|e| anyhow!("Parse JSON error: {} ", e.to_string()))?;
    let data = flatten(&json);

    let mut keys: Vec<&String> = data.keys().collect();
    keys.sort_by_key(|a| a.to_lowercase());

    Ok(keys
        .into_iter()
        .flat_map(|key| value_into_vec(data.get(key).unwrap()))
        .collect())
}

fn value_into_vec(value: &Value) -> Vec<String> {
    match value {
        Value::Array(vec) => {
            let vec_len = vec.len();

            let mut str_vec = Vec::with_capacity(vec_len + 1);

            str_vec.push(vec_len.to_string());
            str_vec.extend(vec.iter().map(to_string_unqoted));

            str_vec
        }
        value => vec![to_string_unqoted(value)],
    }
}

fn to_string_unqoted(value: &impl ToString) -> String {
    let value = value.to_string();
    let mut string = value.as_str();

    if string.starts_with('"') {
        string = &string[1..];
    }

    if string.ends_with('"') {
        string = &string[..string.len() - 1];
    }

    string.to_owned()
}

fn parse(felt_str: &str) -> Result<Felt252, ParseFeltError> {
    Felt252::infer_format_and_parse(&felt_str.replace("\\n", "\n"))
}

#[cfg(test)]
mod tests {
    use super::json_values_sorted_by_keys;

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
        let result = json_values_sorted_by_keys(string).unwrap();
        let expected_result = ["1", "2", "12", "43", "Joh"].to_vec();

        assert_eq!(result, expected_result);

        let string = r#"
        {
            "ad": "string",
            "test": ["1",2,"3",4]
        }"#;
        let result = json_values_sorted_by_keys(string).unwrap();
        let expected_result = ["string", "4", "1", "2", "3", "4"];
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_json_values_sorted_by_keys_invalid_data() {
        let string = r"
        [1,2,'3232']";
        let result = json_values_sorted_by_keys(string);
        assert!(result.is_err());

        let string = r#"
        {
            "test": 'invalid json format'
        }"#;
        let result = json_values_sorted_by_keys(string);
        assert!(result.is_err());
    }
}
