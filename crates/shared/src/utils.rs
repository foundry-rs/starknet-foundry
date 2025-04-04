use cairo_lang_runner::casm_run::format_next_item;
use starknet_types_core::felt::Felt;

/// Helper function to build readable text from a run data.
#[must_use]
pub fn build_readable_text(data: &[Felt]) -> Option<String> {
    let mut data_iter = data.iter().copied();
    let mut items = Vec::new();

    while let Some(item) = format_next_item(&mut data_iter) {
        items.push(item.quote_if_string());
    }

    if items.is_empty() {
        return None;
    }

    let string = if let [item] = &items[..] {
        item.clone()
    } else {
        format!("({})", items.join(", "))
    };

    let mut result = indent_string(&format!("\n{string}"));
    result.push('\n');
    Some(result)
}

fn indent_string(string: &str) -> String {
    let mut modified_string = string.to_string();
    let trailing_newline = if string.ends_with('\n') {
        modified_string.pop();
        true
    } else {
        false
    };

    modified_string = modified_string.replace('\n', "\n    ");
    if trailing_newline {
        modified_string.push('\n');
    }

    modified_string
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
