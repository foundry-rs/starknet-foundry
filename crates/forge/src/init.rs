use anyhow::{anyhow, Context, Ok, Result};

use include_dir::{include_dir, Dir};

use forge::CAIRO_EDITION;
use scarb_api::ScarbCommand;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use toml_edit::{value, ArrayOfTables, DocumentMut, Item, Table};

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
        let contents = replace_project_name(contents, project_name)?;

        fs::write(path, contents)?;
    }

    Ok(())
}

fn replace_project_name(contents: &[u8], project_name: &str) -> Result<Vec<u8>> {
    let contents = std::str::from_utf8(contents).context("UTF-8 error")?;
    let contents = contents.replace("{{ PROJECT_NAME }}", project_name);
    Ok(contents.into_bytes())
}

fn update_config(config_path: &Path) -> Result<()> {
    let config_file = fs::read_to_string(config_path)?;
    let mut document = config_file
        .parse::<DocumentMut>()
        .context("invalid document")?;

    add_target_to_toml(&mut document);
    set_cairo_edition(&mut document, CAIRO_EDITION);
    add_test_script(&mut document);

    fs::write(config_path, document.to_string())?;

    Ok(())
}

fn add_test_script(document: &mut DocumentMut) {
    let mut test = Table::new();

    test.insert("test", value("snforge test"));
    document.insert("scripts", Item::Table(test));
}

fn add_target_to_toml(document: &mut DocumentMut) {
    let mut array_of_tables = ArrayOfTables::new();
    let mut sierra = Table::new();
    let mut contract = Table::new();
    contract.set_implicit(true);

    sierra.insert("sierra", Item::Value(true.into()));
    array_of_tables.push(sierra);
    contract.insert("starknet-contract", Item::ArrayOfTables(array_of_tables));

    document.insert("target", Item::Table(contract));
}

fn set_cairo_edition(document: &mut DocumentMut, cairo_edition: &str) {
    document["package"]["edition"] = value(cairo_edition);
}

fn extend_gitignore(path: &Path) -> Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path.join(".gitignore"))?;
    writeln!(file, ".snfoundry_cache/")?;

    Ok(())
}

trait ScarbCommandExt {
    fn maybe_set_scarb_path(&mut self, path: Option<impl Into<PathBuf>>) -> &mut Self;
}

impl ScarbCommandExt for ScarbCommand {
    fn maybe_set_scarb_path(&mut self, path: Option<impl Into<PathBuf>>) -> &mut Self {
        if let Some(path) = path {
            self.scarb_path(path);
        }
        self
    }
}

pub fn run(project_name: &str) -> Result<()> {
    let project_path = std::env::current_dir()?.join(project_name);

    let scarb = std::env::var("SCARB").ok();

    // if there is no Scarb.toml run `scarb new`
    if project_path.join("Scarb.toml").is_file() {
        ScarbCommand::new_with_stdio()
            .current_dir(std::env::current_dir().context("Failed to get current directory")?)
            .arg("new")
            .arg(&project_path)
            .run()
            .context("Failed to initialize a new project")?;
    }

    let version = env!("CARGO_PKG_VERSION");

    ScarbCommand::new_with_stdio()
        .maybe_set_scarb_path(scarb.clone())
        .current_dir(&project_path)
        .offline()
        .arg("add")
        .arg("--dev")
        .arg("snforge_std")
        .arg("--git")
        .arg("https://github.com/foundry-rs/starknet-foundry.git")
        .arg("--tag")
        .arg(format!("v{version}"))
        .run()
        .context("Failed to add snforge_std")?;

    let cairo_version = ScarbCommand::version().run()?.cairo;

    ScarbCommand::new_with_stdio()
        .maybe_set_scarb_path(scarb)
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
