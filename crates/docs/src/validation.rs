use crate::snippet::{Snippet, SnippetConfig, SnippetType};
use crate::utils::get_nth_ancestor;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::{fs, io, path::Path};
use toml_edit::DocumentMut;

const EXTENSION: Option<&str> = Some("md");

fn load_book_variables() -> HashMap<String, String> {
    let book_toml_path = get_nth_ancestor(2).join("docs/book.toml");
    let content = fs::read_to_string(&book_toml_path).expect("Failed to read book.toml");
    let doc = content
        .parse::<DocumentMut>()
        .expect("Failed to parse book.toml");

    let variables = doc
        .get("preprocessor")
        .and_then(|preprocessor| preprocessor.get("variables"))
        .and_then(|variables| variables.get("variables"))
        .and_then(|variables| variables.as_table());

    let Some(variables) = variables else {
        return HashMap::new();
    };

    variables
        .iter()
        .filter_map(|(key, value)| {
            value
                .as_str()
                .map(|value| (key.to_string(), value.to_string()))
        })
        .collect()
}

fn substitute_book_variables(text: &str, variables: &HashMap<String, String>) -> String {
    let mut result = text.to_string();
    for (key, value) in variables {
        result = result.replace(&format!("{{{{{key}}}}}"), value);
    }
    result
}

pub fn extract_snippets_from_file(
    file_path: &Path,
    snippet_type: &SnippetType,
) -> io::Result<Vec<Snippet>> {
    let book_variables = load_book_variables();
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
                .name("config")
                .map_or_else(String::new, |m| m.as_str().to_string());
            let command_match = caps.name("command")?;
            let output = caps.name("output").map(|m| {
                static GAS_RE: LazyLock<Regex> =
                    LazyLock::new(|| Regex::new(r"gas: (?:~\d+|\{.+\})").unwrap());
                static EXECUTION_RESOURCES_RE: LazyLock<Regex> = LazyLock::new(|| {
                    Regex::new(r"(steps|memory holes|builtins|syscalls|sierra gas): (\d+|\(.+\))")
                        .unwrap()
                });

                let output = GAS_RE.replace_all(m.as_str(), "gas: [..]").to_string();
                let output = EXECUTION_RESOURCES_RE
                    .replace_all(output.as_str(), "${1}: [..]")
                    .to_string();
                substitute_book_variables(&output, &book_variables)
            });

            let config = if config_str.is_empty() {
                SnippetConfig::default()
            } else {
                serde_json::from_str(&config_str).expect("Failed to parse snippet config")
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

        if EXTENSION
            .is_none_or(|ext| path.extension().and_then(|path_ext| path_ext.to_str()) == Some(ext))
        {
            let snippets = extract_snippets_from_file(path, snippet_type)?;
            all_snippets.extend(snippets);
        }
    }

    Ok(all_snippets)
}
