use std::sync::LazyLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

static RE_SNCAST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new( r"(?ms)^(?:<!--\s*(?P<config>\{.*?\})\s*-->\n)?```shell\n\$ (?P<command>sncast .+?)\n```(?:\s*<details>\n<summary>Output:<\/summary>\n\n```shell\n(?P<output>[\s\S]+?)\n```[\s]*<\/details>)?").expect("Failed to create regex for sncast snippet")
});

static RE_SNFORGE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new( r"(?ms)^(?:<!--\s*(?P<config>\{.*?\})\s*-->\n)?```shell\n\$ (?P<command>snforge .+?)\n```(?:\s*<details>\n<summary>Output:<\/summary>\n\n```shell\n(?P<output>[\s\S]+?)\n```[\s]*<\/details>)?").expect("Failed to create regex for snforge snippet")
});

#[derive(Clone, Debug)]
pub struct SnippetType(String);

impl SnippetType {
    #[must_use]
    pub fn forge() -> Self {
        SnippetType("snforge".to_string())
    }

    #[must_use]
    pub fn sncast() -> Self {
        SnippetType("sncast".to_string())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn get_re(&self) -> &'static Regex {
        // The regex pattern is used to match the snippet, its config and the output. Example:
        // <!-- { content of snippet config JSON } -->
        // ```shell
        // $ snforge or sncast command with args...
        // ```
        // <details>
        // <summary>Output:</summary>
        // ```shell
        // Output of the command...
        // ```
        // </details>

        match self.as_str() {
            "snforge" => &RE_SNFORGE,
            "sncast" => &RE_SNCAST,
            _ => panic!("Regex for {} not found", self.as_str()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct SnippetConfig {
    pub ignored: Option<bool>,
    pub package_name: Option<String>,
    pub contract_name: Option<String>,
    pub ignored_output: Option<bool>,
}

#[derive(Debug)]
pub struct Snippet {
    pub command: String,
    pub output: Option<String>,
    pub file_path: String,
    pub line_start: usize,
    pub snippet_type: SnippetType,
    pub config: SnippetConfig,
}

impl Snippet {
    #[must_use]
    pub fn to_command_args(&self) -> Vec<String> {
        let cleaned_command = self
            .command
            .lines()
            .collect::<Vec<&str>>()
            .join(" ")
            .replace(" \\", "");

        shell_words::split(&cleaned_command)
            .expect("Failed to parse snippet string")
            .into_iter()
            .map(|arg| arg.trim().to_string())
            .collect()
    }

    #[must_use]
    pub fn capture_package_from_output(&self) -> Option<String> {
        let re =
            Regex::new(r"Collected \d+ test\(s\) from ([a-zA-Z_][a-zA-Z0-9_]*) package").unwrap();

        re.captures_iter(self.output.as_ref()?)
            .filter_map(|caps| caps.get(1))
            .last()
            .map(|m| m.as_str().to_string())
    }
}
