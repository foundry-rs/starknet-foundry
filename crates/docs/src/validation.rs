use std::{fs, io, path::Path};

use crate::snippet::{Snippet, SnippetConfig, SnippetType};

const EXTENSION: Option<&str> = Some("md");

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

        if EXTENSION.map_or(true, |ext| {
            path.extension().and_then(|path_ext| path_ext.to_str()) == Some(ext)
        }) {
            let snippets = extract_snippets_from_file(path, snippet_type)?;
            all_snippets.extend(snippets);
        }
    }

    Ok(all_snippets)
}
