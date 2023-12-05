use anyhow::{anyhow, Context, Ok, Result};
use camino::Utf8PathBuf;
use std::fs;

use clap::Args;
use indoc::{formatdoc, indoc};
use scarb_metadata::{MetadataCommand, ScarbCommand};

#[derive(Args, Debug)]
pub struct Init {
    /// Name of a script to create
    pub script_name: String,
}

pub fn init(init_args: &Init) -> Result<()> {
    let script_root_dir_path = get_script_root_dir_path(&init_args.script_name)?;

    init_scarb_project(&script_root_dir_path, &init_args.script_name)?;
    add_dependencies(&script_root_dir_path)?;
    modify_files_in_src_dir(&script_root_dir_path, &init_args.script_name)
}

fn get_script_root_dir_path(script_name: &str) -> Result<String> {
    let current_dir = std::env::current_dir()?;

    let script_root_dir_path = current_dir
        .file_name()
        .and_then(|dir_name| dir_name.to_str())
        .filter(|&dir_name| dir_name == "scripts")
        .map_or_else(
            || current_dir.join("scripts").join(script_name),
            |_| current_dir.join(script_name),
        );

    script_root_dir_path
        .to_str()
        .ok_or_else(|| anyhow!("Failed to get script root dir"))
        .map(|s| s.to_owned())
}

fn init_scarb_project(script_root_dir: &str, script_name: &str) -> Result<()> {
    ScarbCommand::new()
        .args(["new", "--name", &script_name, "--no-vcs", &script_root_dir])
        .run()
        .context("Failed to init Scarb project")?;

    Ok(())
}

fn add_dependencies(script_root_dir: &str) -> Result<()> {
    add_sncast_std_dependency(script_root_dir)
        .context("Failed to add sncast_std dependency to Scarb.toml")?;
    add_starknet_dependency(script_root_dir)
        .context("Failed to add starknet dependency to Scarb.toml")?;

    Ok(())
}

fn add_sncast_std_dependency(script_root_dir: &str) -> Result<()> {
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

fn add_starknet_dependency(script_root_dir: &str) -> Result<()> {
    let cairo_version =
        get_cairo_version(script_root_dir).context("Failed to obtain cairo version")?;
    let starknet_dependency = format!("starknet@>={cairo_version}");

    ScarbCommand::new()
        .current_dir(script_root_dir)
        .args(["--offline", "add", &starknet_dependency])
        .run()?;

    Ok(())
}

fn get_cairo_version(path: &str) -> Result<String> {
    let scarb_metadata = MetadataCommand::new().current_dir(path).no_deps().exec()?;

    Ok(scarb_metadata.app_version_info.cairo.version.to_string())
}

fn modify_files_in_src_dir(script_root_dir: &str, script_name: &str) -> Result<()> {
    create_script_main_file(script_root_dir, script_name)
        .context(format!("Failed to create {}.cairo file", script_name))?;
    overwrite_lib_file(script_root_dir, script_name).context("Failed to overwrite lib.cairo file")
}

fn create_script_main_file(script_root_dir: &str, script_name: &str) -> Result<()> {
    let script_main_file_name = format!("{script_name}.cairo");
    let script_main_file_path = Utf8PathBuf::from(script_root_dir)
        .join("src")
        .join(script_main_file_name);

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

fn overwrite_lib_file(script_root_dir: &str, script_name: &str) -> Result<()> {
    let lib_file_path = Utf8PathBuf::from(script_root_dir)
        .join("src")
        .join("lib.cairo");

    fs::write(
        lib_file_path,
        formatdoc! {r#"
            mod {script_name};
        "#},
    )?;

    Ok(())
}
