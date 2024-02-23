use regex::Regex;
use snapbox::cmd::OutputAssert;
use std::borrow::Cow;

pub trait AsOutput {
    fn as_stdout(&self) -> Cow<'_, str>;
    fn as_stderr(&self) -> Cow<'_, str>;
}

impl AsOutput for OutputAssert {
    fn as_stdout(&self) -> Cow<'_, str> {
        String::from_utf8(self.get_output().stdout.clone())
            .unwrap()
            .into()
    }
    fn as_stderr(&self) -> Cow<'_, str> {
        String::from_utf8(self.get_output().stderr.clone())
            .unwrap()
            .into()
    }
}

impl AsOutput for String {
    fn as_stdout(&self) -> Cow<'_, str> {
        self.into()
    }
    fn as_stderr(&self) -> Cow<'_, str> {
        self.into()
    }
}

fn find_with_wildcard(line: &str, actual: &[String]) -> Option<usize> {
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

    let mut contains = true;
    let mut out = String::new();

    for line in &asserted_lines {
        if is_present(line, &mut actual_lines) {
            out.push_str(&format!("| {line}\n"));
        } else {
            contains = false;
            out.push_str(&format!("- {line}\n"));
        }
    }

    if !contains {
        actual_lines
            .iter()
            .for_each(|line| out.push_str(&format!("+ {line}\n")));
    }

    assert!(contains, "Output does not match:\n\n{out}");
}

#[allow(clippy::needless_pass_by_value)]
pub fn assert_stdout_contains(output: impl AsOutput, lines: impl AsRef<str>) {
    let stdout = output.as_stdout();

    assert_output_contains(&stdout, lines.as_ref());
}

#[allow(clippy::needless_pass_by_value)]
pub fn assert_stderr_contains(output: impl AsOutput, lines: impl AsRef<str>) {
    let stderr = output.as_stderr();

    assert_output_contains(&stderr, lines.as_ref());
}
