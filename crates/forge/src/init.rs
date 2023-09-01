use anyhow::{bail, Result};





use std::ffi::OsStr;
use std::path::{Path};


/// Returns `true` if the name contains non-ASCII characters.
pub fn is_non_ascii_name(name: &str) -> bool {
    name.chars().any(|ch| ch > '\x7f')
}

/// These names cannot be used on Windows, even with an extension.
pub fn is_windows_reserved(name: &str) -> bool {
    [
        "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
        "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
    ]
    .contains(&name.to_ascii_lowercase().as_str())
}

pub fn is_reserved_keyword(name: &str) -> bool {
    ["_"].contains(&name.to_ascii_lowercase().as_str())
}

fn copy_recursively(
    source: impl AsRef<Path>,
    destination: impl AsRef<Path>,
) -> std::io::Result<()> {
    std::fs::create_dir_all(&destination)?;
    for entry in std::fs::read_dir(source)? {
        let entry = entry?;
        let filetype = entry.file_type()?;
        if filetype.is_dir() {
            copy_recursively(entry.path(), destination.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), destination.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
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

/// See also `util::toml::embedded::sanitize_name`
fn check_name(name: &str) -> Result<()> {
    if is_windows_reserved(name) {
        if cfg!(windows) {
            bail!(
                "cannot use name `{}`, it is a reserved Windows filename",
                name
            );
        } else {
            bail!(format!(
                "the name `{}` is a reserved Windows filename\n\
                This package will not work on Windows platforms.",
                name
            ));
        }
    }
    if is_non_ascii_name(name) {
        bail!(format!(
            "the name `{}` contains non-ASCII characters.",
            name
        ));
    }

    if is_reserved_keyword(name) {
        bail!(format!("the name `{}` is reserved keyword", name));
    }

    if name.chars().next().unwrap().is_numeric() {
        bail!(format!("the name `{}` starts with digit.", name));
    }

    Ok(())
}

pub fn init(name: Option<String>) -> Result<()> {
    let project_name = name.unwrap_or("starknet_forge_template".to_string());
    check_name(&project_name)?;
    let project_path = std::env::current_dir().unwrap().join(project_name);
    check_path(project_path.as_path())?;

    if project_path.exists() {
        bail!(
            "Destination {} already exists.\n
            New project couldn't be created",
            &project_path.display().to_string()
        )
    }

    let template_path = project_root::get_project_root()
        .unwrap()
        .join("starknet_forge_template");
    copy_recursively(template_path, project_path.display().to_string())?;

    Ok(())
}
