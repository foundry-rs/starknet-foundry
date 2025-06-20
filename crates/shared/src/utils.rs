use starknet_api::execution_utils::format_panic_data;
use starknet_types_core::felt::Felt;

/// Helper function to build readable text from run data.
#[must_use]
pub fn build_readable_text(data: &[Felt]) -> Option<String> {
    if data.is_empty() {
        return None;
    }

    let string = format_panic_data(data);

    let mut result = indent_string(&format!("\n{string}"));
    result.push('\n');
    Some(result)
}

fn indent_string(string: &str) -> String {
    let without_trailing = string.strip_suffix('\n').unwrap_or(string);
    let indented = without_trailing.replace('\n', "\n    ");
    let should_append_newline = string.ends_with('\n');

    if should_append_newline {
        format!("{indented}\n")
    } else {
        indented
    }
}

#[cfg(test)]
mod tests {
    use super::indent_string;

    #[test]
    fn test_indent_string() {
        let s = indent_string("\nabc\n");
        assert_eq!(s, "\n    abc\n");

        let s = indent_string("\nabc");
        assert_eq!(s, "\n    abc");

        let s = indent_string("\nabc\nd");
        assert_eq!(s, "\n    abc\n    d");
    }
}
