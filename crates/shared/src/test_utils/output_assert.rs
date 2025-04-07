use regex::Regex;
use snapbox::cmd::OutputAssert;
use std::fmt::Write as _;

pub trait AsOutput {
    fn as_stdout(&self) -> &str;
    fn as_stderr(&self) -> &str;
}

impl AsOutput for OutputAssert {
    fn as_stdout(&self) -> &str {
        std::str::from_utf8(&self.get_output().stdout).unwrap()
    }
    fn as_stderr(&self) -> &str {
        std::str::from_utf8(&self.get_output().stderr).unwrap()
    }
}

impl AsOutput for String {
    fn as_stdout(&self) -> &str {
        self
    }
    fn as_stderr(&self) -> &str {
        self
    }
}

fn find_with_wildcard(line: &str, actual: &[&str]) -> Option<usize> {
    let escaped = regex::escape(line);
    let replaced = escaped.replace("\\[\\.\\.\\]", ".*");
    let wrapped = format!("^{replaced}$");
    let re = Regex::new(wrapped.as_str()).unwrap();

    actual.iter().position(|other| re.is_match(other))
}

fn is_present(line: &str, actual: &mut Vec<&str>) -> bool {
    let position = find_with_wildcard(line, actual);
    if let Some(position) = position {
        actual.remove(position);
        return true;
    }
    false
}

fn assert_output_contains(output: &str, lines: &str) {
    let asserted_lines = lines.lines();
    let mut actual_lines: Vec<&str> = output.lines().collect();

    let mut contains = true;
    let mut out = String::new();

    for line in asserted_lines {
        if is_present(line, &mut actual_lines) {
            writeln!(out, "| {line}").unwrap();
        } else {
            contains = false;
            writeln!(out, "- {line}").unwrap();
        }
    }

    if !contains {
        for line in &actual_lines {
            writeln!(out, "+ {line}").unwrap();
        }
    }

    assert!(contains, "Output does not match:\n\n{out}");
}
fn assert_output_contains_exact(output: &str, lines: &str) {
    let converted_pattern = regex::escape(lines).replace(r"\[\.\.\]", ".*");
    let re = Regex::new(&converted_pattern).unwrap();
    assert!(
        re.is_match(output),
        "Pattern not found in output. Expected pattern:\n{lines}",
    );
}

#[expect(clippy::needless_pass_by_value)]
pub fn assert_stdout_contains(output: impl AsOutput, lines: impl AsRef<str>) {
    let stdout = output.as_stdout();

    assert_output_contains(stdout, lines.as_ref());
}

#[expect(clippy::needless_pass_by_value)]
pub fn assert_stderr_contains(output: impl AsOutput, lines: impl AsRef<str>) {
    let stderr = output.as_stderr();

    assert_output_contains(stderr, lines.as_ref());
}

#[expect(clippy::needless_pass_by_value)]
pub fn assert_stdout_contains_exact(output: impl AsOutput, lines: impl AsRef<str>) {
    let stdout = output.as_stdout();

    assert_output_contains_exact(stdout, lines.as_ref());
}
