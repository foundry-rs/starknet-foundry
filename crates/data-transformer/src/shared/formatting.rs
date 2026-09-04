use std::fmt::Display;

use itertools::Itertools;
use starknet_rust::core::types::contract::AbiNamedMember;

const MAX_INLINE_ARGUMENTS_WIDTH: usize = 100;

#[derive(Clone, Copy)]
pub enum ArgumentListKind {
    Positional,
}

impl ArgumentListKind {
    const fn delimiters(self) -> (char, char) {
        match self {
            Self::Positional => ('(', ')'),
        }
    }
}

/// Formats ABI members as individual `name: type` entries.
pub fn format_abi_members(members: &[AbiNamedMember]) -> Vec<String> {
    members
        .iter()
        .map(|member| format!("{}: {}", member.name, member.r#type))
        .collect()
}

/// Formats an argument error with optional diagnostics and a shared passed/expected layout.
pub fn format_passed_vs_expected<P, E, D>(
    headline: impl Display,
    diagnostics: &[D],
    passed: &[P],
    expected: &[E],
    list_kind: ArgumentListKind,
) -> String
where
    P: AsRef<str>,
    E: AsRef<str>,
    D: AsRef<str>,
{
    let mut output = headline.to_string();

    for diagnostic in diagnostics {
        output.push_str("\n  ");
        output.push_str(diagnostic.as_ref());
    }

    if !diagnostics.is_empty() {
        output.push('\n');
    }

    output.push('\n');
    output.push_str(&format_argument_list("passed", passed, list_kind));
    output.push('\n');
    output.push_str(&format_argument_list("expected", expected, list_kind));
    output
}

fn format_argument_list<T: AsRef<str>>(
    label: &str,
    arguments: &[T],
    list_kind: ArgumentListKind,
) -> String {
    let (opening, closing) = list_kind.delimiters();

    if arguments.is_empty() {
        return format!("  {label}: {opening}{closing}");
    }

    let inline_arguments = arguments.iter().map(AsRef::as_ref).join(", ");
    let inline = format!("  {label}: {opening}{inline_arguments}{closing}");

    if !inline.contains('\n') && inline.chars().count() <= MAX_INLINE_ARGUMENTS_WIDTH {
        return inline;
    }

    let multiline_arguments = arguments
        .iter()
        .map(AsRef::as_ref)
        .map(|argument| argument.replace('\n', "\n    "))
        .join(",\n    ");

    format!("  {label}: {opening}\n    {multiline_arguments}\n  {closing}")
}

#[cfg(test)]
mod tests {
    use super::{ArgumentListKind, format_passed_vs_expected};

    #[test]
    fn formats_empty_argument_lists_inline() {
        let arguments: [&str; 0] = [];

        assert_eq!(
            format_passed_vs_expected(
                "Invalid arguments",
                &[] as &[&str],
                &arguments,
                &arguments,
                ArgumentListKind::Positional,
            ),
            "Invalid arguments\n  passed: ()\n  expected: ()"
        );
    }

    #[test]
    fn formats_short_argument_lists_inline() {
        assert_eq!(
            format_passed_vs_expected(
                "Invalid arguments",
                &["unexpected argument at position 3"],
                &["1", "2", "3"],
                &["a: felt252", "b: felt252"],
                ArgumentListKind::Positional,
            ),
            "Invalid arguments\n  unexpected argument at position 3\n\n  passed: (1, 2, 3)\n  expected: (a: felt252, b: felt252)"
        );
    }

    #[test]
    fn formats_long_argument_lists_on_separate_lines() {
        assert_eq!(
            format_passed_vs_expected(
                "Invalid arguments",
                &[] as &[&str],
                &["foo", "bar"],
                &[
                    "first: core::array::Array::<core::array::Array::<core::felt252>>",
                    "second: core::array::Array::<core::array::Array::<core::felt252>>",
                ],
                ArgumentListKind::Positional,
            ),
            "Invalid arguments\n  passed: (foo, bar)\n  expected: (\n    first: core::array::Array::<core::array::Array::<core::felt252>>,\n    second: core::array::Array::<core::array::Array::<core::felt252>>\n  )"
        );
    }
}
