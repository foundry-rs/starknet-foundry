use camino::Utf8PathBuf;
use indoc::{formatdoc, indoc};
use scarb_api::ScarbCommand;
use snapbox::cmd::{cargo_bin, Command};
use sncast::helpers::constants::INIT_SCRIPTS_DIR;
use sncast::helpers::scarb_utils::get_cairo_version;
use tempfile::TempDir;

#[test]
fn test_script_init_happy_case() {
    let script_name = "my_script";
    let temp_dir = TempDir::new().expect("Unable to create a temporary directory");

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(temp_dir.path())
        .args(["script", "init", script_name]);

    snapbox.assert().stdout_matches(formatdoc! {r"
        Warning: [..]
        command: script init
        message: Successfully initialized `{script_name}` at [..]/scripts/{script_name}
    "});

    let script_dir_path = temp_dir.path().join(INIT_SCRIPTS_DIR).join(script_name);
    let scarb_toml_path = script_dir_path.join("Scarb.toml");

    let scarb_toml_content = std::fs::read_to_string(&scarb_toml_path).unwrap();
    let lib_cairo_content = std::fs::read_to_string(script_dir_path.join("src/lib.cairo")).unwrap();
    let main_file_content =
        std::fs::read_to_string(script_dir_path.join(format!("src/{script_name}.cairo"))).unwrap();

    let cast_version = env!("CARGO_PKG_VERSION");

    let scarb_toml_path = Utf8PathBuf::from_path_buf(scarb_toml_path).unwrap();
    let cairo_version = get_cairo_version(&scarb_toml_path).unwrap();

    let expected_scarb_toml = formatdoc!(
        r#"
            [package]
            name = "{script_name}"
            version = "0.1.0"
            edition = [..]

            # See more keys and their definitions at https://docs.swmansion.com/scarb/docs/reference/manifest.html

            [dependencies]
            sncast_std = {{ git = "https://github.com/foundry-rs/starknet-foundry", tag = "v{cast_version}" }}
            starknet = ">={cairo_version}"
        "#
    );

    snapbox::assert_matches(expected_scarb_toml, scarb_toml_content);

    assert_eq!(
        lib_cairo_content,
        formatdoc! {r#"
            mod {script_name};
        "#}
    );
    assert_eq!(
        main_file_content,
        indoc! {r#"
            use sncast_std::{call, CallResult};

            // The example below uses a contract deployed to the Goerli testnet
            fn main() {
                let contract_address = 0x7ad10abd2cc24c2e066a2fee1e435cd5fa60a37f9268bfbaf2e98ce5ca3c436;
                let call_result = call(contract_address.try_into().unwrap(), 'get_greeting', array![]);
                assert(*call_result.data[0]=='Hello, Starknet!', *call_result.data[0]);
                println!("{:?}", call_result);
            }
        "#}
    );
}

#[test]
fn test_init_fails_when_scripts_dir_exists_in_cwd() {
    let script_name = "my_script";
    let temp_dir = TempDir::new().expect("Unable to create a temporary directory");

    std::fs::create_dir_all(temp_dir.path().join(INIT_SCRIPTS_DIR))
        .expect("Failed to create scripts directory in the current temp directory");

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(temp_dir.path())
        .args(["script", "init", script_name]);

    snapbox.assert().stderr_matches(formatdoc! {r"
        command: script init
        error: Scripts directory already exists at [..]
    "});
}

#[test]
fn test_init_twice_fails() {
    let script_name = "my_script";
    let temp_dir = TempDir::new().expect("Unable to create a temporary directory");

    Command::new(cargo_bin!("sncast"))
        .current_dir(temp_dir.path())
        .args(["script", "init", script_name])
        .assert()
        .success();

    assert!(temp_dir.path().join(INIT_SCRIPTS_DIR).exists());

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(temp_dir.path())
        .args(["script", "init", script_name]);

    snapbox.assert().stderr_matches(formatdoc! {r#"
        command: script init
        error: Scripts directory already exists at [..]
    "#});
}

#[test]
fn test_initialized_script_compiles() {
    let script_name = "my_script";
    let temp_dir = TempDir::new().expect("Unable to create a temporary directory");

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(temp_dir.path())
        .args(["script", "init", script_name]);

    snapbox.assert().stdout_matches(formatdoc! {r"
        Warning: [..]
        command: script init
        message: Successfully initialized `{script_name}` at [..]/scripts/{script_name}
    "});

    let script_dir_path = temp_dir.path().join(INIT_SCRIPTS_DIR).join(script_name);

    ScarbCommand::new_with_stdio()
        .current_dir(script_dir_path)
        .arg("build")
        .run()
        .expect("Failed to compile the initialized script");
}
