use anyhow::Context;
use cairo_felt::Felt252;
use cairo_lang_runner::casm_run::MemBuffer;
use cairo_lang_runner::short_string::as_cairo_short_string;
use cheatnet::cheatcodes::EnhancedHintError;
use cheatnet::cheatcodes::EnhancedHintError::FileParsing;
use num_bigint::BigUint;

pub(super) fn parse_txt(
    buffer: &mut MemBuffer,
    file_path: Felt252,
) -> Result<(), EnhancedHintError> {
    let file_path_str = as_cairo_short_string(&file_path)
        .with_context(|| format!("Failed to convert {file_path} to str"))?;
    let content = std::fs::read_to_string(file_path_str.clone())?;
    let split_content: Vec<&str> = content.trim().split_ascii_whitespace().collect();

    let felts_in_results: Vec<Result<Felt252, ()>> = split_content
        .iter()
        .map(|&string| string_into_felt(string))
        .collect();

    let felts = felts_in_results
        .iter()
        .cloned()
        .collect::<Result<Vec<Felt252>, ()>>()
        .map_err(|_| FileParsing {
            path: file_path_str,
        })?;

    buffer
        .write_data(felts.iter())
        .expect("Failed to insert file content to memory");
    Ok(())
}

fn string_into_felt(string: &str) -> Result<Felt252, ()> {
    let maybe_number = string.parse::<BigUint>();
    match maybe_number {
        Ok(number) => Ok(number.into()),
        Err(_) => {
            let length = string.len();
            let first_char = string.chars().nth(0);
            let last_char = string.chars().nth(length - 1);

            if length >= 2
                && length - 2 <= 31
                && (first_char == Some('\'') || first_char == Some('\"'))
                && first_char == last_char
                && string.is_ascii()
            {
                let string = string.to_string();
                let bytes = string[1..length - 1].as_bytes();
                Ok(Felt252::from_bytes_be(bytes))
            } else {
                Err(())
            }
        }
    }
}
