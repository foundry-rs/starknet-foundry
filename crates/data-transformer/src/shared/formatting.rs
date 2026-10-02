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
pub fn format_passed_vs_expected(
    headline: &str,
    diagnostics: &[String],
    passed: &[String],
    expected: &[String],
    list_kind: ArgumentListKind,
) -> String {
    let mut output = headline.to_owned();

    for diagnostic in diagnostics {
        output.push_str("\n  ");
        output.push_str(diagnostic);
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

fn format_argument_list(label: &str, arguments: &[String], list_kind: ArgumentListKind) -> String {
    let (opening, closing) = list_kind.delimiters();

    if arguments.is_empty() {
        return format!("  {label}: {opening}{closing}");
    }

    let inline_arguments = arguments.iter().join(", ");
    let inline = format!("  {label}: {opening}{inline_arguments}{closing}");

    if !inline.contains('\n') && inline.chars().count() <= MAX_INLINE_ARGUMENTS_WIDTH {
        return inline;
    }

    let multiline_arguments = arguments
        .iter()
        .map(|argument| argument.replace('\n', "\n    "))
        .join(",\n    ");

    format!("  {label}: {opening}\n    {multiline_arguments}\n  {closing}")
}
