use anyhow::{bail, Context, Ok, Result};

use include_dir::{include_dir, Dir, DirEntry};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use toml_edit::{ArrayOfTables, Document, Item, Table};

static TEMPLATE: Dir = include_dir!("starknet_forge_template");

fn overwrite_files_from_template(
    dir_to_overwrite: &str,
    base_path: &PathBuf,
    project_name: &str,
) -> Result<()> {
    let copy_form_dir = TEMPLATE.get_dir(dir_to_overwrite);
    match copy_form_dir {
        Some(dir) => {
            for file in dir.files() {
                fs::create_dir_all(base_path.join(Path::new(dir_to_overwrite)))?;
                let path = base_path.join(file.path());
                let contents = file.contents();
                let contents = replace_project_name(contents, project_name);

                fs::write(path, contents)?;
            }
        }
        None => {}
    }

    Ok(())
}

fn replace_project_name(contents: &[u8], project_name: &str) -> Vec<u8> {
    // SAFETY: We control these files, and we know that they are UTF-8.
    let contents = unsafe { std::str::from_utf8_unchecked(contents) };
    let contents = contents.replace("{{ PROJECT_NAME }}", project_name);
    contents.into_bytes()
}

fn extend_scarb_toml(path: &PathBuf) -> Result<()> {
    dbg!(&path);
    let config_file = fs::read_to_string(path)?;

    let mut doc = config_file.parse::<Document>().expect("invalid document");

    let mut array = ArrayOfTables::new();
    let mut table = Table::new();
    let mut table2 = Table::new();
    table.insert("casm", Item::Value(true.into()));
    table2.insert("starknet-contract", Item::Table(table));
    array.push(table2);
    dbg!(&array);

    doc.insert("target", Item::ArrayOfTables(array));
    fs::write(path, doc.to_string())?;
    Ok(())
}

// [[target.starknet-contract]]
// casm = true

pub fn init(name: Option<String>) -> Result<()> {
    let project_name = name.unwrap_or("starknet_forge_template".to_string());
    let project_path = std::env::current_dir()?.join(&project_name);

    Command::new("scarb")
        .current_dir(std::env::current_dir().context("Failed to get current directory")?)
        .arg("new")
        .arg(&project_path)
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .context("Failed to initial new project")?;

    Command::new("scarb")
        .current_dir(&project_path)
        .arg("add")
        .arg("snforge_std")
        .arg("--git")
        .arg("https://github.com/foundry-rs/starknet-foundry.git")
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .context("Failed to add snforge_std")?;

    Command::new("scarb")
        .current_dir(&project_path)
        .arg("add")
        .arg("starknet@2.2.0")
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .context("Failed to add starknet")?;
    extend_scarb_toml(&project_path.join("Scarb.toml"))?;
    overwrite_files_from_template("src", &project_path, &project_name)?;
    overwrite_files_from_template("tests", &project_path, &project_name)?;

    Ok(())
}
