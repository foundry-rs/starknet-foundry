use anyhow::{anyhow, Context, Ok, Result};
use camino::Utf8PathBuf;
use std::fs;

use cast::helpers::constants::SCRIPTS_DIR;
use cast::helpers::response_structs::ScriptInitResponse;
use cast::helpers::scarb_utils::get_cairo_version;
use clap::Args;
use indoc::{formatdoc, indoc};
use scarb_metadata::ScarbCommand;

#[derive(Args, Debug)]
pub struct Init {
    /// Name of a script to create
    pub script_name: String,
}

pub fn init(init_args: &Init) -> Result<ScriptInitResponse> {
    let script_root_dir_path = get_script_root_dir_path(&init_args.script_name)?;

    init_scarb_project(&init_args.script_name, &script_root_dir_path)?;
    add_dependencies(&script_root_dir_path)?;
    modify_files_in_src_dir(&init_args.script_name, &script_root_dir_path)?;

    Ok(ScriptInitResponse {
        status: format!("Successfully initialized `{}`", init_args.script_name),
    })
}

fn get_script_root_dir_path(script_name: &str) -> Result<Utf8PathBuf> {
    let current_dir = std::env::current_dir()?;

    let script_root_dir_path = current_dir
        .file_name()
        .and_then(|dir_name| dir_name.to_str())
        .filter(|&dir_name| dir_name == SCRIPTS_DIR)
        .map_or_else(
            || current_dir.join(SCRIPTS_DIR).join(script_name),
            |_| current_dir.join(script_name),
        );

    Utf8PathBuf::from_path_buf(script_root_dir_path)
        .map_err(|_| anyhow!("Failed to obtain script root dir path"))
}

fn init_scarb_project(script_name: &str, script_root_dir: &Utf8PathBuf) -> Result<()> {
    ScarbCommand::new()
        .args([
            "new",
            "--name",
            &script_name,
            "--no-vcs",
            "--quiet",
            script_root_dir.as_str(),
        ])
        .run()
        .context("Failed to init Scarb project")?;

    Ok(())
}

fn add_dependencies(script_root_dir: &Utf8PathBuf) -> Result<()> {
    add_sncast_std_dependency(script_root_dir)
        .context("Failed to add sncast_std dependency to Scarb.toml")?;
    add_starknet_dependency(script_root_dir)
        .context("Failed to add starknet dependency to Scarb.toml")?;

    Ok(())
}

fn add_sncast_std_dependency(script_root_dir: &Utf8PathBuf) -> Result<()> {
    let cast_version = env!("CARGO_PKG_VERSION");
    let cast_version = &format!("v{cast_version}");

    ScarbCommand::new()
        .current_dir(script_root_dir)
        .args([
            "--offline",
            "add",
            "sncast_std",
            "--git",
            "https://github.com/foundry-rs/starknet-foundry.git",
            "--tag",
            &cast_version,
        ])
        .run()?;

    Ok(())
}

fn add_starknet_dependency(script_root_dir: &Utf8PathBuf) -> Result<()> {
    let scarb_manifest_path = script_root_dir.join("Scarb.toml");
    let cairo_version =
        get_cairo_version(&scarb_manifest_path).context("Failed to obtain cairo version")?;
    let starknet_dependency = format!("starknet@>={cairo_version}");

    ScarbCommand::new()
        .current_dir(script_root_dir)
        .args(["--offline", "add", &starknet_dependency])
        .run()?;

    Ok(())
}

fn modify_files_in_src_dir(script_name: &str, script_root_dir: &Utf8PathBuf) -> Result<()> {
    create_script_main_file(script_name, script_root_dir)
        .context(format!("Failed to create {}.cairo file", script_name))?;
    overwrite_lib_file(script_name, script_root_dir).context("Failed to overwrite lib.cairo file")
}

fn create_script_main_file(script_name: &str, script_root_dir: &Utf8PathBuf) -> Result<()> {
    let script_main_file_name = format!("{script_name}.cairo");
    let script_main_file_path = script_root_dir.join("src").join(script_main_file_name);

    fs::write(
        script_main_file_path,
        indoc! {r#"
            use sncast_std;
            use debug::PrintTrait;

            fn main() {
                'Put your code here!'.print();
            }
        "#},
    )?;

    Ok(())
}

fn overwrite_lib_file(script_name: &str, script_root_dir: &Utf8PathBuf) -> Result<()> {
    let lib_file_path = script_root_dir.join("src/lib.cairo");

    fs::write(
        lib_file_path,
        formatdoc! {r#"
            mod {script_name};
        "#},
    )?;

    Ok(())
}
