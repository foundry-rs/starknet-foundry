use crate::snippet::Snippet;
use anyhow::{anyhow, Result};
use camino::Utf8PathBuf;
use std::{env, fs, path::PathBuf, str::FromStr};
use tempfile::TempDir;
use toml_edit::{value, DocumentMut};

#[must_use]
pub fn get_nth_ancestor(levels_up: usize) -> PathBuf {
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

pub fn print_snippets_validation_summary(snippets: &[Snippet], tool_name: &str) {
    let validated_snippets_count = snippets
        .iter()
        .filter(|snippet| !snippet.config.ignored)
        .count();
    let ignored_snippets_count = snippets.len() - validated_snippets_count;

    println!("Finished validation of {tool_name} docs snippets\nValidated: {validated_snippets_count}, Ignored: {ignored_snippets_count}");
}

pub fn print_ignored_snippet_message(snippet: &Snippet) {
    println!(
        "Ignoring {} docs snippet, file: {}:{}:1",
        snippet.snippet_type.as_str(),
        snippet.file_path,
        snippet.line_start,
    );
}

fn get_canonical_path(relative_path: &str) -> Result<String> {
    Ok(Utf8PathBuf::from_str(relative_path)
        .map_err(|e| anyhow!("Failed to create Utf8PathBuf: {}", e))?
        .canonicalize_utf8()
        .map_err(|e| anyhow!("Failed to canonicalize path: {}", e))?
        .to_string())
}

pub fn update_scarb_toml_dependencies(temp: &TempDir) -> Result<(), Box<dyn std::error::Error>> {
    let snforge_std_path = get_canonical_path("../../snforge_std")?;
    let sncast_std_path = get_canonical_path("../../sncast_std")?;
    let scarb_toml_path = temp.path().join("Scarb.toml");

    let mut scarb_toml = fs::read_to_string(&scarb_toml_path)
        .unwrap()
        .parse::<DocumentMut>()
        .unwrap();

    scarb_toml["dependencies"]["sncast_std"]["path"] = value(&sncast_std_path);
    scarb_toml["dev-dependencies"]["snforge_std"]["path"] = value(&snforge_std_path);

    fs::write(&scarb_toml_path, scarb_toml.to_string())?;

    Ok(())
}
