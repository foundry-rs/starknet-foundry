use anyhow::{anyhow, Context, Ok, Result};

use include_dir::{include_dir, Dir};

use regex::Regex;
use scarb_api::ScarbCommand;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::str::from_utf8;
use toml_edit::{value, ArrayOfTables, Document, Item, Table};

const CAIRO_EDITION: &str = "2023_10";

static TEMPLATE: Dir = include_dir!("starknet_forge_template");

fn overwrite_files_from_scarb_template(
    dir_to_overwrite: &str,
    base_path: &Path,
    project_name: &str,
) -> Result<()> {
    let copy_from_dir = TEMPLATE.get_dir(dir_to_overwrite).ok_or_else(|| {
        anyhow!(
            "Directory {} doesn't exist in the template.",
            dir_to_overwrite
        )
    })?;

    for file in copy_from_dir.files() {
        fs::create_dir_all(base_path.join(Path::new(dir_to_overwrite)))?;
        let path = base_path.join(file.path());
        let contents = file.contents();
        let contents = replace_project_name(contents, project_name);

        fs::write(path, contents)?;
    }

    Ok(())
}

fn replace_project_name(contents: &[u8], project_name: &str) -> Vec<u8> {
    let contents = std::str::from_utf8(contents).expect("UTF-8 error");
    let contents = contents.replace("{{ PROJECT_NAME }}", project_name);
    contents.into_bytes()
}

fn update_config(config_path: &Path) -> Result<()> {
    let config_file = fs::read_to_string(config_path)?;
    let mut document = config_file.parse::<Document>().expect("invalid document");

    add_target_to_toml(&mut document);
    set_cairo_edition(&mut document, CAIRO_EDITION);

    fs::write(config_path, document.to_string())?;

    Ok(())
}

fn add_target_to_toml(document: &mut Document) {
    let mut array_of_tables = ArrayOfTables::new();
    let mut casm = Table::new();
    let mut contract = Table::new();
    contract.set_implicit(true);

    casm.insert("casm", Item::Value(true.into()));
    array_of_tables.push(casm);
    contract.insert("starknet-contract", Item::ArrayOfTables(array_of_tables));
    document.insert("target", Item::Table(contract));
}

fn set_cairo_edition(document: &mut Document, cairo_edition: &str) {
    document["package"]["edition"] = value(cairo_edition);
}

fn extend_gitignore(path: &Path) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path.join(".gitignore"))?;
    writeln!(file, ".snfoundry_cache/")?;

    Ok(())
}

pub fn run(project_name: &str) -> Result<()> {
    let project_path = std::env::current_dir()?.join(project_name);

    ScarbCommand::new_with_stdio()
        .current_dir(std::env::current_dir().context("Failed to get current directory")?)
        .arg("new")
        .arg(&project_path)
        .run()
        .context("Failed to initialize a new project")?;

    let version = env!("CARGO_PKG_VERSION");

    ScarbCommand::new_with_stdio()
        .current_dir(&project_path)
        .offline()
        .arg("add")
        .arg("snforge_std")
        .arg("--git")
        .arg("https://github.com/foundry-rs/starknet-foundry.git")
        .arg("--tag")
        .arg(format!("v{version}"))
        .run()
        .context("Failed to add snforge_std")?;

    let scarb_version = ScarbCommand::new()
        .arg("--version")
        .inherit_stderr()
        .command()
        .output()
        .context("Failed to execute `scarb --version`")?;
    let version_output = from_utf8(&scarb_version.stdout)
        .context("Failed to parse `scarb --version` output to UTF-8")?;

    let cairo_version_regex = Regex::new(r"(?:cairo:?\s*)([0-9]+.[0-9]+.[0-9]+)")
        .expect("Could not create cairo version matching regex");
    let cairo_version_capture = cairo_version_regex
        .captures(version_output)
        .expect("Could not find cairo version");
    let cairo_version = cairo_version_capture
        .get(1)
        .expect("Could not find cairo version")
        .as_str();

    ScarbCommand::new_with_stdio()
        .current_dir(&project_path)
        .offline()
        .arg("add")
        .arg(format!("starknet@{cairo_version}"))
        .run()
        .context("Failed to add starknet")?;

    update_config(&project_path.join("Scarb.toml"))?;
    extend_gitignore(&project_path)?;
    overwrite_files_from_scarb_template("src", &project_path, project_name)?;
    overwrite_files_from_scarb_template("tests", &project_path, project_name)?;

    Ok(())
}
