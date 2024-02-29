use anyhow::{anyhow, bail, Result};
use cairo_felt::Felt252;
use flatten_serde_json::flatten;
use num_bigint::BigUint;
use runtime::EnhancedHintError;
use serde_json::{Map, Value};

pub(super) fn read_txt(file_path: String) -> Result<Vec<Felt252>, EnhancedHintError> {
    let content = std::fs::read_to_string(&file_path)?;

    let mut result = vec![];

    for felt_str in content.trim().split_ascii_whitespace() {
        match string_into_felt(felt_str) {
            Ok(felt) => result.push(felt),
            Err(_) => return Err(EnhancedHintError::FileParsing { path: file_path }),
        }
    }

    Ok(result)
}

pub(super) fn read_json(file_path: String) -> Result<Vec<Felt252>, EnhancedHintError> {
    let content = std::fs::read_to_string(&file_path)?;
    let split_content = json_values_sorted_by_keys(&content)
        .map_err(|e| anyhow!("{}, in file {}", e.to_string(), file_path))?;

    let mut result = Vec::with_capacity(split_content.len());

    for felt_str in &split_content {
        match string_into_felt(felt_str) {
            Ok(felt) => result.push(felt),
            Err(_) => return Err(EnhancedHintError::FileParsing { path: file_path }),
        }
    }

    Ok(result)
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
            let vec_len = vec.len().to_string();

            let mut str_vec = vec![];

            str_vec.push(vec_len);
            str_vec.extend(vec.iter().map(ToString::to_string));

            str_vec
        }
        value => vec![value.to_string()],
    }
}

pub(super) fn string_into_felt(string: &str) -> Result<Felt252> {
    if let Ok(number) = string.parse::<BigUint>() {
        // By default it is replaced with 0 in this case
        if number < Felt252::prime() {
            Ok(number.into())
        } else {
            bail!("Number = {number} is too big to fit in a felt252")
        }
    } else {
        let length = string.len();
        let first_char = string.chars().next();
        let last_char = string.chars().nth(length - 1);
        if length >= 2
            && length - 2 <= 31
            && (first_char == Some('\'') || first_char == Some('\"'))
            && first_char == last_char
            && string.is_ascii()
        {
            let bytes = string[1..length - 1].as_bytes();
            Ok(Felt252::from_bytes_be(bytes))
        } else {
            bail!("Failed to parse value = {string} to short string")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::runtime_extensions::forge_runtime_extension::file_operations::string_into_felt;
    use cairo_felt::Felt252;
    use num_bigint::BigUint;
    use num_traits::One;

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
        let expected_result = ["1", "2", "12", "43", "\"Joh\""].to_vec();

        assert_eq!(result, expected_result);

        let string = r#"
        {
            "ad": "string",
            "test": ["1",2,"3",4]
        }"#;
        let result = json_values_sorted_by_keys(string).unwrap();
        let expected_result = ["\"string\"", "4", "\"1\"", "2", "\"3\"", "4"];
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

    #[test]
    fn test_string_into_felt_positive() {
        let string = "123";
        assert_eq!(string_into_felt(string).unwrap(), 123.into());
    }

    #[test]
    fn test_string_into_felt_negative() {
        let string = "-123";
        assert!(string_into_felt(string).is_err());
    }

    #[test]
    fn test_string_into_felt_prime_minus_one() {
        let string = "3618502788666131213697322783095070105623107215331596699973092056135872020480";
        let expected = Felt252::prime() - BigUint::one();
        assert_eq!(string_into_felt(string).unwrap(), expected.into());
    }

    #[test]
    fn test_string_into_felt_prime() {
        let string = "3618502788666131213697322783095070105623107215331596699973092056135872020481";
        assert!(string_into_felt(string).is_err());
    }

    #[test]
    fn test_string_into_felt_nan() {
        let string = "A2bA";
        assert!(string_into_felt(string).is_err());
    }

    #[test]
    fn test_string_into_felt_shortstring() {
        let string = "\'1he5llo9\'";
        let string2 = "\"1he5llo9\"";
        let felt = string_into_felt(string).unwrap();
        let felt2 = string_into_felt(string2).unwrap();
        assert_eq!(felt, felt2);
        assert_eq!(
            felt,
            Felt252::from_bytes_be(string[1..string.len() - 1].as_bytes())
        );
    }

    #[test]
    fn test_string_into_felt_shortstring_mismatched_quotes() {
        let string = "\'1he5llo9\"";
        assert!(string_into_felt(string).is_err());
        let string = "\"1he5llo9\'";
        assert!(string_into_felt(string).is_err());
    }

    #[test]
    fn test_string_into_felt_shortstring_missing_quote() {
        let string = "\'1he5llo9";
        assert!(string_into_felt(string).is_err());
    }

    #[test]
    fn test_string_into_felt_shortstring_empty() {
        let string = "\'\'";
        assert_eq!(string_into_felt(string).unwrap(), 0.into());
    }

    #[test]
    fn test_string_into_felt_shortstring_too_long() {
        let string = "\'abcdefghjiklmnoprstqwyzabcdefghi\'";
        assert!(string_into_felt(string).is_err());
    }

    #[test]
    fn test_string_into_felt_shortstring_non_ascii() {
        let string = "\'abcd§g\'";
        assert!(string_into_felt(string).is_err());
    }
}
