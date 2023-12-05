use cast::helpers::constants::SCRIPTS_DIR;
use indoc::{formatdoc, indoc};
use snapbox::cmd::{cargo_bin, Command};
use tempfile::TempDir;

#[test]
fn test_init_files_content() {
    let script_name = "myscript";
    let temp_dir = TempDir::new().expect("Unable to create a temporary directory");
    let script_dir_path = temp_dir.path().join(SCRIPTS_DIR).join(script_name);

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(temp_dir.path())
        .args(["script", "init", script_name]);

    snapbox.assert().stdout_eq(formatdoc! {r"
        command: script init
        status: Successfully initialized `{script_name}`
    "});

    let scarb_toml_content = std::fs::read_to_string(script_dir_path.join("Scarb.toml")).unwrap();
    let lib_cairo_content = std::fs::read_to_string(script_dir_path.join("src/lib.cairo")).unwrap();
    let main_file_content =
        std::fs::read_to_string(script_dir_path.join(format!("src/{}.cairo", script_name)))
            .unwrap();

    let cast_version = env!("CARGO_PKG_VERSION");
    let expected_scarb_toml = formatdoc!(
        r#"
            [package]
            name = "{script_name}"
            version = "0.1.0"

            # See more keys and their definitions at https://docs.swmansion.com/scarb/docs/reference/manifest.html

            [dependencies]
            sncast_std = {{ git = "https://github.com/foundry-rs/starknet-foundry", tag = "v{}" }}
            starknet = ">=2.3.1"
        "#,
        cast_version
    );

    assert_eq!(scarb_toml_content, expected_scarb_toml);
    assert_eq!(
        lib_cairo_content,
        formatdoc! {r#"
            mod {script_name};
        "#}
    );
    assert_eq!(
        main_file_content,
        indoc! {r#"
            use sncast_std;
            use debug::PrintTrait;

            fn main() {
                'Put your code here!'.print();
            }
        "#}
    );
}

#[test]
fn test_init_creates_scripts_dir() {
    let script_name = "myscript";
    let temp_dir = TempDir::new().expect("Unable to create a temporary directory");

    assert!(
        !temp_dir.path().join(SCRIPTS_DIR).exists(),
        "Scripts directory already exists in the current temp directory"
    );

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(temp_dir.path())
        .args(["script", "init", script_name]);

    snapbox.assert().stdout_eq(formatdoc! {r"
        command: script init
        status: Successfully initialized `{script_name}`
    "});

    assert!(temp_dir.path().join(SCRIPTS_DIR).exists());
    assert!(temp_dir.path().join(SCRIPTS_DIR).join(script_name).exists());
}

#[test]
fn test_init_from_scripts_dir() {
    let script_name = "myscript";
    let temp_dir = TempDir::new().expect("Unable to create a temporary directory");
    let scripts_dir_path = temp_dir.path().join(SCRIPTS_DIR);

    std::fs::create_dir_all(&scripts_dir_path)
        .expect("Failed to create scripts directory in the current temp directory");
    assert!(scripts_dir_path.exists());

    let snapbox = Command::new(cargo_bin!("sncast"))
        .current_dir(temp_dir.path())
        .args(["script", "init", script_name]);

    snapbox.assert().stdout_eq(formatdoc! {r"
        command: script init
        status: Successfully initialized `{script_name}`
    "});

    assert!(scripts_dir_path.join(script_name).exists());
    assert!(!scripts_dir_path
        .join(SCRIPTS_DIR)
        .join(script_name)
        .exists());
}
