use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

const EXTENSION: Option<&str> = Some("md");

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
    pub fn get_re(&self) -> Regex {
        // The regex pattern is used to match the snippet, its config and the output. Example:
        // <!-- { "ignored": true, "package_name": "xyz" } -->
        // ```shell
        // $ <snforge or sncast command with args>
        // ```
        // <details>
        // <summary>Output:</summary>
        // ```shell
        // <output>
        // ```
        // </details>

        let escaped_command = regex::escape(self.as_str());
        let pattern = format!(
            r"(?ms)^(?:<!--\s*(.*?)\s*-->\n)?```shell\n\$ ({escaped_command} .+?)\n```(?:\s*<details>\n<summary>Output:<\/summary>\n\n```shell\n([\s\S]+?)\n```[\s]*<\/details>)?"
        );

        Regex::new(&pattern).unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct SnippetConfig {
    pub ignored: Option<bool>,
    pub package_name: Option<String>,
}

impl SnippetConfig {
    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }

    fn default() -> Self {
        SnippetConfig {
            ignored: None,
            package_name: None,
        }
    }
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
    pub fn to_command_args(&self) -> Vec<String> {
        let cleaned_command = self
            .command
            .lines()
            .map(str::trim_end)
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

pub fn extract_snippets_from_file(
    file_path: &Path,
    snippet_type: &SnippetType,
) -> io::Result<Vec<Snippet>> {
    let content = fs::read_to_string(file_path)?;
    let file_path_str = file_path
        .to_str()
        .expect("Failed to get file path")
        .to_string();

    let snippets = snippet_type
        .get_re()
        .captures_iter(&content)
        .filter_map(|caps| {
            let match_start = caps.get(0)?.start();
            let config_str = caps
                .get(1)
                .map_or_else(String::new, |m| m.as_str().to_string());
            let command_match = caps.get(2)?;
            let output = caps.get(3).map(|m| m.as_str().to_string());

            let config = if config_str.is_empty() {
                SnippetConfig::default()
            } else {
                SnippetConfig::from_json(&config_str).expect("Failed to parse snippet config")
            };

            Some(Snippet {
                command: command_match.as_str().to_string(),
                output,
                file_path: file_path_str.clone(),
                line_start: content[..match_start].lines().count() + 1,
                snippet_type: snippet_type.clone(),
                config,
            })
        })
        .collect();

    Ok(snippets)
}

pub fn extract_snippets_from_directory(
    dir_path: &Path,
    snippet_type: &SnippetType,
) -> io::Result<Vec<Snippet>> {
    let mut all_snippets = Vec::new();

    let files = walkdir::WalkDir::new(dir_path)
        .into_iter()
        .map(|entry| entry.expect("Failed to read directory"))
        .filter(|entry| entry.path().is_file());

    for file in files {
        let path = file.path();

        if EXTENSION.map_or(true, |ext| {
            path.extension().and_then(|path_ext| path_ext.to_str()) == Some(ext)
        }) {
            let snippets = extract_snippets_from_file(path, snippet_type)?;
            all_snippets.extend(snippets);
        }
    }

    Ok(all_snippets)
}

#[must_use]
pub fn get_parent_dir(levels_up: usize) -> PathBuf {
    let mut dir = env::current_dir().expect("Failed to get the current directory");

    for _ in 0..levels_up {
        dir = dir
            .parent()
            .expect("Failed to navigate to parent directory")
            .to_owned();
    }

    dir
}

pub fn assert_valid_snippet(condition: bool, snippet: &Snippet, err_message: &str) {
    assert!(
        condition,
        "Found invalid {} snippet in the docs at {}:{}:1\n{}",
        snippet.snippet_type.as_str(),
        snippet.file_path,
        snippet.line_start,
        err_message
    );
}

pub fn print_success_message(snippets_len: usize, tool_name: &str) {
    println!("Successfully validated {snippets_len} {tool_name} docs snippets");
}

pub fn print_skipped_snippet_message(snippet: &Snippet) {
    println!(
        "Skipped validation of {} snippet in the docs in file: {} at line {}",
        snippet.snippet_type.as_str(),
        snippet.file_path,
        snippet.line_start,
    );
}
