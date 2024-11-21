use regex::Regex;
use std::{
    collections::HashMap,
    env, fs, io,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

const EXTENSION: Option<&str> = Some("md");

#[derive(Clone, Debug)]
pub enum SnippetType {
    Forge,
    Sncast,
}

impl SnippetType {
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            SnippetType::Forge => "snforge",
            SnippetType::Sncast => "sncast",
        }
    }

    #[must_use]
    pub fn get_re(&self) -> Regex {
        let pattern = format!(
            r"(?ms)^(?:<!--\s*(.*?)\s*-->\n)?```shell\n\$ ({} .+?)\n```(?:\s*<details>\n<summary>Output:<\/summary>\n\n```shell\n([\s\S]+?)\n```[\s]*<\/details>)?",
            self.as_str()
        );

        Regex::new(&pattern).unwrap()
    }
}

#[derive(Debug)]
pub struct Snippet {
    pub command: String,
    pub output: Option<String>,
    pub file_path: String,
    pub line_start: usize,
    pub snippet_type: SnippetType,
    pub ignored: bool,
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
            let command_match = caps.get(2)?;
            let match_start = caps.get(0)?.start();
            let output = caps.get(3).map(|m| m.as_str().to_string());
            let ignored = caps.get(1).map_or(false, |m| m.as_str().contains("ignore"));

            Some(Snippet {
                command: command_match.as_str().to_string(),
                output,
                file_path: file_path_str.clone(),
                line_start: content[..match_start].lines().count() + 1,
                snippet_type: snippet_type.clone(),
                ignored,
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
        "Found invalid {} snippet in the docs in at {}:{}:1\n{}",
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

pub fn create_listings_to_packages_mapping() -> HashMap<String, Vec<String>> {
    let docs_listings_path = "../../docs/listings";
    let mut mapping = HashMap::new();

    for listing in WalkDir::new(docs_listings_path)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_dir())
    {
        let listing_path = listing.path();
        let crates_dir = listing_path.join("crates");

        if crates_dir.is_dir() {
            let packages = list_packages_in_directory(&crates_dir);
            if let Some(listing_name) = listing_path.file_name().and_then(|x| x.to_str()) {
                if !packages.is_empty() {
                    mapping.insert(listing_name.to_string(), packages);
                }
            }
        }
    }

    mapping
}

fn list_packages_in_directory(dir_path: &Path) -> Vec<String> {
    let crates_path = dir_path.join("");

    if crates_path.exists() && crates_path.is_dir() {
        WalkDir::new(crates_path)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir())
            .filter_map(|e| {
                e.path()
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(String::from)
            })
            .collect()
    } else {
        Vec::new()
    }
}
