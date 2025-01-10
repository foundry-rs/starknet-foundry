use anyhow::{anyhow, ensure, Context, Ok, Result};
use camino::Utf8PathBuf;
use std::fs;

use clap::Args;
use indoc::{formatdoc, indoc};
use scarb_api::ScarbCommand;
use shared::print::print_as_warning;
use sncast::helpers::constants::INIT_SCRIPTS_DIR;
use sncast::helpers::scarb_utils::get_cairo_version;
use sncast::response::structs::ScriptInitResponse;

#[derive(Args, Debug)]
pub struct Init {
    /// Name of a script to create
    pub script_name: String,
}

pub fn init(init_args: &Init) -> Result<ScriptInitResponse> {
    let script_root_dir_path = get_script_root_dir_path(&init_args.script_name)?;

    init_scarb_project(&init_args.script_name, &script_root_dir_path)?;

    let modify_files_result = add_dependencies(&script_root_dir_path)
        .and_then(|()| modify_files_in_src_dir(&init_args.script_name, &script_root_dir_path));

    print_as_warning(&anyhow!(
        "The newly created script isn't auto-added to the workspace. For more details, please see https://foundry-rs.github.io/starknet-foundry/starknet/script.html#initialize-a-script")
    );

    match modify_files_result {
        Result::Ok(()) => Ok(ScriptInitResponse {
            message: format!(
                "Successfully initialized `{}` at {}",
                init_args.script_name, script_root_dir_path
            ),
        }),
        Err(err) => {
            clean_created_dir_and_files(&script_root_dir_path);
            Err(err)
        }
    }
}

fn get_script_root_dir_path(script_name: &str) -> Result<Utf8PathBuf> {
    let current_dir = Utf8PathBuf::from_path_buf(std::env::current_dir()?)
        .expect("Failed to create Utf8PathBuf for the current directory");

    let scripts_dir = current_dir.join(INIT_SCRIPTS_DIR);

    ensure!(
        !scripts_dir.exists(),
        "Scripts directory already exists at `{scripts_dir}`"
    );

    Ok(scripts_dir.join(script_name))
}

fn init_scarb_project(script_name: &str, script_root_dir: &Utf8PathBuf) -> Result<()> {
    ScarbCommand::new()
        .args([
            "new",
            "--name",
            script_name,
            "--no-vcs",
            "--quiet",
            script_root_dir.as_str(),
            "--test-runner",
            "cairo-test",
        ])
        .env("SCARB_INIT_TEST_RUNNER", "cairo-test")
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
    let cast_version = format!("v{}", env!("CARGO_PKG_VERSION"));

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
        get_cairo_version(&scarb_manifest_path).context("Failed to get cairo version")?;
    let starknet_dependency = format!("starknet@>={cairo_version}");

    ScarbCommand::new()
        .current_dir(script_root_dir)
        .args(["--offline", "add", &starknet_dependency])
        .run()?;

    Ok(())
}

fn modify_files_in_src_dir(script_name: &str, script_root_dir: &Utf8PathBuf) -> Result<()> {
    create_script_main_file(script_name, script_root_dir)
        .context(format!("Failed to create {script_name}.cairo file"))?;
    overwrite_lib_file(script_name, script_root_dir).context("Failed to overwrite lib.cairo file")
}

fn create_script_main_file(script_name: &str, script_root_dir: &Utf8PathBuf) -> Result<()> {
    let script_main_file_name = format!("{script_name}.cairo");
    let script_main_file_path = script_root_dir.join("src").join(script_main_file_name);

    fs::write(
        script_main_file_path,
        indoc! {r#"
            use sncast_std::{call, CallResult};

            // The example below uses a contract deployed to the Sepolia testnet
            fn main() {
                let contract_address = 0x07e867f1fa6da2108dd2b3d534f1fbec411c5ec9504eb3baa1e49c7a0bef5ab5;
                let call_result = call(contract_address.try_into().unwrap(), selector!("get_greeting"), array![]).expect('call failed');
                assert(*call_result.data[1]=='Hello, Starknet!', *call_result.data[1]);
                println!("{:?}", call_result);
            }
        "#},
    )?;

    Ok(())
}

fn overwrite_lib_file(script_name: &str, script_root_dir: &Utf8PathBuf) -> Result<()> {
    let lib_file_path = script_root_dir.join("src/lib.cairo");

    fs::write(
        lib_file_path,
        formatdoc! {r"
            mod {script_name};
        "},
    )?;

    Ok(())
}

fn clean_created_dir_and_files(script_root_dir: &Utf8PathBuf) {
    if fs::remove_dir_all(script_root_dir).is_err() {
        eprintln!("Failed to clean created files by init command at {script_root_dir}");
    }
}
