use crate::scarb::config::SCARB_MANIFEST_TEMPLATE_CONTENT;
use crate::{NewArgs, CAIRO_EDITION};
use anyhow::{anyhow, bail, ensure, Context, Ok, Result};
use camino::Utf8PathBuf;
use include_dir::{include_dir, Dir};
use indoc::formatdoc;
use scarb_api::ScarbCommand;
use semver::Version;
use shared::consts::FREE_RPC_PROVIDER_URL;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use toml_edit::{value, ArrayOfTables, DocumentMut, Item, Table};

static TEMPLATE: Dir = include_dir!("starknet_forge_template");

const DEFAULT_ASSERT_MACROS: Version = Version::new(0, 1, 0);
const MINIMAL_SCARB_FOR_CORRESPONDING_ASSERT_MACROS: Version = Version::new(2, 8, 0);

fn create_snfoundry_manifest(path: &PathBuf) -> Result<()> {
    fs::write(
        path,
        formatdoc! {r#"
        # Visit https://foundry-rs.github.io/starknet-foundry/appendix/snfoundry-toml.html
        # and https://foundry-rs.github.io/starknet-foundry/projects/configuration.html for more information

        # [sncast.default]                                         # Define a profile name
        # url = "{default_rpc_url}" # Url of the RPC provider
        # accounts-file = "../account-file"                        # Path to the file with the account data
        # account = "mainuser"                                     # Account from `accounts_file` or default account file that will be used for the transactions
        # keystore = "~/keystore"                                  # Path to the keystore file
        # wait-params = {{ timeout = 300, retry-interval = 10 }}     # Wait for submitted transaction parameters
        # block-explorer = "StarkScan"                             # Block explorer service used to display links to transaction details
        # show-explorer-links = true                               # Print links pointing to pages with transaction details in the chosen block explorer
        "#,
            default_rpc_url = FREE_RPC_PROVIDER_URL,
        },
    )?;

    Ok(())
}

fn add_template_to_scarb_manifest(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        bail!("Scarb.toml not found");
    }

    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .context("Failed to open Scarb.toml")?;

    file.write_all(SCARB_MANIFEST_TEMPLATE_CONTENT.as_bytes())
        .context("Failed to write to Scarb.toml")?;
    Ok(())
}

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

fn update_config(config_path: &Path, scarb: &Version) -> Result<()> {
    let config_file = fs::read_to_string(config_path)?;
    let mut document = config_file
        .parse::<DocumentMut>()
        .context("invalid document")?;

    add_target_to_toml(&mut document);
    set_cairo_edition(&mut document, CAIRO_EDITION);
    add_test_script(&mut document);
    add_assert_macros(&mut document, scarb)?;

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

fn add_assert_macros(document: &mut DocumentMut, scarb: &Version) -> Result<()> {
    let version = if scarb < &MINIMAL_SCARB_FOR_CORRESPONDING_ASSERT_MACROS {
        &DEFAULT_ASSERT_MACROS
    } else {
        scarb
    };

    document
        .get_mut("dev-dependencies")
        .and_then(|dep| dep.as_table_mut())
        .context("Failed to get dev-dependencies from Scarb.toml")?
        .insert("assert_macros", value(version.to_string()));

    Ok(())
}

fn extend_gitignore(path: &Path) -> Result<()> {
    if path.join(".gitignore").exists() {
        let mut file = OpenOptions::new()
            .append(true)
            .open(path.join(".gitignore"))?;
        writeln!(file, ".snfoundry_cache/")?;
    }
    Ok(())
}

pub fn new(
    NewArgs {
        path,
        name,
        no_vcs,
        overwrite,
    }: NewArgs,
) -> Result<()> {
    if !overwrite {
        ensure!(
            !path.exists() || path.read_dir().is_ok_and(|mut i| i.next().is_none()),
            format!("The provided path `{path}` points to a non-empty directory. If you wish to create a project in this directory, please use the `--overwrite` flag to proceed.")
        );
    }
    let name = infer_name(name, &path)?;

    fs::create_dir_all(&path)?;
    let project_path = path.canonicalize()?;
    let scarb_manifest_path = project_path.join("Scarb.toml");
    let snfoundry_manifest_path = project_path.join("snfoundry.toml");

    // if there is no Scarb.toml run `scarb init`
    if !scarb_manifest_path.is_file() {
        let mut cmd = ScarbCommand::new_with_stdio();
        cmd.current_dir(&project_path)
            .args(["init", "--name", &name]);

        if no_vcs {
            cmd.arg("--no-vcs");
        }

        cmd.env("SCARB_INIT_TEST_RUNNER", "cairo-test")
            .run()
            .context("Failed to initialize a new project")?;

        ScarbCommand::new_with_stdio()
            .current_dir(&project_path)
            .manifest_path(scarb_manifest_path.clone())
            .offline()
            .arg("remove")
            .arg("--dev")
            .arg("cairo_test")
            .run()
            .context("Failed to remove cairo_test dependency")?;
    }

    add_template_to_scarb_manifest(&scarb_manifest_path)?;

    if !snfoundry_manifest_path.is_file() {
        create_snfoundry_manifest(&snfoundry_manifest_path)?;
    }

    let version = env!("CARGO_PKG_VERSION");
    let cairo_version = ScarbCommand::version().run()?.cairo;

    if env::var("DEV_DISABLE_SNFORGE_STD_DEPENDENCY").is_err() {
        ScarbCommand::new_with_stdio()
            .current_dir(&project_path)
            .manifest_path(scarb_manifest_path.clone())
            .offline()
            .arg("add")
            .arg("--dev")
            .arg(format!("snforge_std@{version}"))
            .run()
            .context("Failed to add snforge_std dependency")?;
    }

    ScarbCommand::new_with_stdio()
        .current_dir(&project_path)
        .manifest_path(scarb_manifest_path.clone())
        .offline()
        .arg("add")
        .arg(format!("starknet@{cairo_version}"))
        .run()
        .context("Failed to add starknet dependency")?;

    update_config(&project_path.join("Scarb.toml"), &cairo_version)?;
    extend_gitignore(&project_path)?;
    overwrite_files_from_scarb_template("src", &project_path, &name)?;
    overwrite_files_from_scarb_template("tests", &project_path, &name)?;

    // Fetch to create lock file.
    ScarbCommand::new_with_stdio()
        .manifest_path(scarb_manifest_path)
        .arg("fetch")
        .run()
        .context("Failed to fetch created project")?;

    Ok(())
}

fn infer_name(name: Option<String>, path: &Utf8PathBuf) -> Result<String> {
    let name = if let Some(name) = name {
        name
    } else {
        let Some(file_name) = path.file_name() else {
            bail!("Cannot infer package name from path: {path}. Please: use the flag `--name`");
        };
        file_name.to_string()
    };

    Ok(name)
}
