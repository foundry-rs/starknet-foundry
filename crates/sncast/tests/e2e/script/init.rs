use crate::helpers::constants::devnet_url;
use crate::helpers::runner::runner;
use camino::Utf8PathBuf;
use indoc::{formatdoc, indoc};
use scarb_api::ScarbCommand;
use shared::test_utils::output_assert::{assert_stderr_contains, assert_stdout_contains};
use sncast::helpers::constants::INIT_SCRIPTS_DIR;
use sncast::helpers::scarb_utils::get_cairo_version;
use tempfile::TempDir;

#[test]
fn test_script_init_happy_case() {
    let script_name = "my_script";
    let temp_dir = TempDir::new().expect("Unable to create a temporary directory");

    let snapbox = runner(&["script", "init", script_name]).current_dir(temp_dir.path());

    snapbox.assert().stdout_matches(formatdoc! {r"
        [WARNING] [..]
        Success: Script initialization completed
        
        Initialized `{script_name}` at [..]/scripts/{script_name}
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
            sncast_std = "{cast_version}"
            starknet = ">={cairo_version}"

            [dev-dependencies]
            cairo_test = "{cairo_version}"
        "#
    );

    snapbox::assert_matches(expected_scarb_toml, scarb_toml_content);

    assert_eq!(
        lib_cairo_content,
        formatdoc! {r"
            mod {script_name};
        "}
    );
    assert_eq!(
        main_file_content,
        indoc! {r#"
            use sncast_std::call;

            // The example below uses a contract deployed to the Sepolia testnet
            const CONTRACT_ADDRESS: felt252 =
                0x07e867f1fa6da2108dd2b3d534f1fbec411c5ec9504eb3baa1e49c7a0bef5ab5;

            fn main() {
                let call_result = call(
                    CONTRACT_ADDRESS.try_into().unwrap(), selector!("get_greeting"), array![],
                )
                    .expect('call failed');

                assert(*call_result.data[1] == 'Hello, Starknet!', *call_result.data[1]);

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

    let snapbox = runner(&["script", "init", script_name]).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: script init
        Error: Scripts directory already exists at [..]
        "},
    );
}

#[test]
fn test_init_twice_fails() {
    let script_name = "my_script";
    let temp_dir = TempDir::new().expect("Unable to create a temporary directory");

    let url = devnet_url();
    let args = vec!["script", "init", script_name];
    runner(&args)
        .current_dir(temp_dir.path())
        .assert()
        .success();

    assert!(temp_dir.path().join(INIT_SCRIPTS_DIR).exists());

    let snapbox = runner(&args).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stderr_contains(
        output,
        indoc! {r"
        Command: script init
        Error: Scripts directory already exists at [..]
        "},
    );
}

#[ignore = "Fails if plugin is unreleased, fixed and restore after release"]
#[test]
fn test_initialized_script_compiles() {
    let script_name = "my_script";
    let temp_dir = TempDir::new().expect("Unable to create a temporary directory");

    let snapbox = runner(&["script", "init", script_name]).current_dir(temp_dir.path());
    let output = snapbox.assert().success();

    assert_stdout_contains(
        output,
        formatdoc! {r"
        [WARNING] The newly created script isn't auto-added to the workspace. [..]
        Success: Script initialization completed
        
        Initialized `{script_name}` at [..]{script_name}
    "},
    );

    let script_dir_path = temp_dir.path().join(INIT_SCRIPTS_DIR).join(script_name);

    // Using a tag during the release process will cause the test to fail as the new tag won't exist in the repository yet
    // This command will overwrite sncast_std dependency to use the master branch instead of a tag
    ScarbCommand::new_with_stdio()
        .current_dir(&script_dir_path)
        .args([
            "--offline",
            "add",
            "sncast_std",
            "--git",
            "https://github.com/foundry-rs/starknet-foundry.git",
            "--branch",
            "master",
        ])
        .run()
        .expect("Failed to overwrite sncast_std dependency in Scarb.toml");

    ScarbCommand::new_with_stdio()
        .current_dir(&script_dir_path)
        .arg("build")
        .run()
        .expect("Failed to compile the initialized script");
}
