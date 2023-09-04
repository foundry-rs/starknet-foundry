use anyhow::{bail, Result};

use include_dir::{include_dir, Dir, DirEntry};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

static TEMPLATE: Dir = include_dir!("starknet_forge_template");

fn create_with_template(dir: &Dir<'_>, base_path: &PathBuf, project_name: &str) -> Result<()> {
    fs::create_dir_all(base_path)?;

    for entry in dir.entries() {
        let path = base_path.join(entry.path());

        match entry {
            DirEntry::Dir(d) => {
                fs::create_dir_all(&path)?;
                create_with_template(d, base_path, project_name)?;
            }
            DirEntry::File(f) => {
                let contents = f.contents();
                let contents = replace_project_name(contents, project_name);

                fs::write(path, contents)?;
            }
        }
    }

    Ok(())
}

fn replace_project_name(contents: &[u8], project_name: &str) -> Vec<u8> {
    // SAFETY: We control these files, and we know that they are UTF-8.
    let contents = unsafe { std::str::from_utf8_unchecked(contents) };
    let contents = contents.replace("{{ PROJECT_NAME }}", project_name);
    contents.into_bytes()
}

fn check_path(path: &Path) -> Result<()> {
    // warn if the path contains characters that will break `env::join_paths`
    if std::env::join_paths(std::slice::from_ref(&OsStr::new(path))).is_err() {
        let path = path.to_string_lossy();
        bail!(format!(
            "the path `{path}` contains invalid PATH characters (usually `:`, `;`, or `\"`)\n\
            It is recommended to use a different name to avoid problems."
        ));
    }
    Ok(())
}

/// Inspired by scarb package name validation
/// https://github.com/software-mansion/scarb/blob/main/scarb/src/core/package/name.rs#L57
fn check_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("empty string cannot be used as package name");
    }

    if name == "_" {
        bail!("underscore cannot be used as package name");
    }

    if !name.eq(&name.to_ascii_lowercase()) {
        bail!(
            "invalid package name: `{name}`\n\
            note: usage of ASCII uppercase letters in the package name has been disallowed\n\
            help: change package name to: {}",
            name.to_ascii_lowercase()
        )
    }

    let mut chars = name.chars();

    // Validate first letter.
    if let Some(ch) = chars.next() {
        // A specific error for a potentially common case.
        if ch.is_ascii_digit() {
            bail!(
                "the name `{name}` cannot be used as a package name, \
                names cannot start with a digit"
            );
        }

        if !(ch.is_ascii_alphabetic() || ch == '_') {
            bail!(
                "invalid character `{ch}` in package name: `{name}`, \
                the first character must be an ASCII lowercase letter or underscore"
            )
        }
    }

    // Validate rest.
    for ch in chars {
        if !(ch.is_ascii_alphanumeric() || ch == '_') {
            bail!(
                "invalid character `{ch}` in package name: `{name}`, \
                characters must be ASCII lowercase letters, ASCII numbers or underscore"
            )
        }
    }

    Ok(())
}

pub fn init(name: Option<String>) -> Result<()> {
    let project_name = name.unwrap_or("starknet_forge_template".to_string());
    check_name(&project_name)?;
    let project_path = std::env::current_dir().unwrap().join(&project_name);
    check_path(project_path.as_path())?;

    if project_path.exists() {
        bail!(
            "Destination {} already exists.\n
            New project couldn't be created",
            &project_path.display().to_string()
        )
    }

    create_with_template(&TEMPLATE, &project_path, &project_name)?;

    Ok(())
}
