use std::borrow::Cow;

use regex::Regex;
use snapbox::cmd::OutputAssert;

pub trait IntoOutput {
    fn into_stdout<'a>(&'a self) -> Cow<'a, str>;
    fn into_stderr<'a>(&'a self) -> Cow<'a, str>;
}

impl IntoOutput for OutputAssert {
    fn into_stdout<'a>(&'a self) -> Cow<'a, str> {
        String::from_utf8(self.get_output().stdout.clone())
            .unwrap()
            .into()
    }
    fn into_stderr<'a>(&'a self) -> Cow<'a, str> {
        String::from_utf8(self.get_output().stderr.clone())
            .unwrap()
            .into()
    }
}

impl IntoOutput for String {
    fn into_stdout<'a>(&'a self) -> Cow<'a, str> {
        self.into()
    }
    fn into_stderr<'a>(&'a self) -> Cow<'a, str> {
        self.into()
    }
}

fn find_with_wildcard(line: &str, actual: &Vec<String>) -> Option<usize> {
    let escaped = regex::escape(line);
    let replaced = escaped.replace("\\[\\.\\.\\]", ".*");
    let wrapped = format!("^{replaced}$");
    let re = Regex::new(wrapped.as_str()).unwrap();

    actual.iter().position(|other| re.is_match(other))
}

fn is_present(line: &str, actual: &mut Vec<String>) -> bool {
    let position = find_with_wildcard(line, actual);
    if let Some(position) = position {
        actual.remove(position);
        return true;
    }
    false
}

fn assert_output_contains(output: &str, lines: &str) {
    let asserted_lines: Vec<String> = lines.lines().map(std::convert::Into::into).collect();
    let mut actual_lines: Vec<String> = output.lines().map(std::convert::Into::into).collect();

    let mut matches = true;
    let mut out = String::new();

    for line in &asserted_lines {
        if is_present(line, &mut actual_lines) {
            out.push_str("| ");
        } else {
            matches = false;
            out.push_str("- ");
        }
        out.push_str(line);
        out.push('\n');
    }
    for remaining_line in actual_lines {
        matches = false;
        out.push_str("+ ");
        out.push_str(&remaining_line);
        out.push('\n');
    }

    assert!(matches, "Stdout does not match:\n\n{out}");
}

pub fn assert_stdout_contains(output: impl IntoOutput, lines: impl AsRef<str>) {
    let stdout = output.into_stdout();

    assert_output_contains(&stdout, lines.as_ref());
}
pub fn assert_stderr_contains(output: impl IntoOutput, lines: impl AsRef<str>) {
    let stderr = output.into_stderr();

    assert_output_contains(&stderr, lines.as_ref());
}
