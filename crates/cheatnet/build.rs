use std::{env, path::PathBuf, process::Command};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let setup_script = manifest_dir.join("../../scripts/setup_predeployed_contracts.sh");
    let artifacts = [
        "src/data/predeployed_contracts/ERC20Lockable/casm.json.gz",
        "src/data/predeployed_contracts/ERC20Lockable/sierra.json.gz",
        "src/data/predeployed_contracts/ERC20Mintable/casm.json.gz",
        "src/data/predeployed_contracts/ERC20Mintable/sierra.json.gz",
    ];

    let artifact_paths: Vec<_> = artifacts
        .into_iter()
        .map(|relative_path| manifest_dir.join(relative_path))
        .collect();

    for path in &artifact_paths {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    println!("cargo:rerun-if-changed={}", setup_script.display());

    if artifact_paths.iter().all(|path| path.is_file()) {
        return;
    }

    let status = Command::new("bash")
        .arg(&setup_script)
        .current_dir(
            manifest_dir
                .parent()
                .and_then(|path| path.parent())
                .expect("workspace root"),
        )
        .status()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", setup_script.display()));

    assert!(
        status.success(),
        "predeployed contract setup failed with status {status}\n\
run ./scripts/setup_predeployed_contracts.sh manually for details"
    );

    for path in &artifact_paths {
        assert!(
            path.is_file(),
            "missing predeployed contract artifact at {}\n\
setup script completed but did not generate all required files",
            path.display()
        );
    }
}
