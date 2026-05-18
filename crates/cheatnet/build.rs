use std::{env, path::Path, process::Command};

const REQUIRED_ARTIFACTS: &[&str] = &[
    "src/data/predeployed_contracts/ERC20Lockable/casm.json.gz",
    "src/data/predeployed_contracts/ERC20Lockable/sierra.json.gz",
    "src/data/predeployed_contracts/ERC20Mintable/casm.json.gz",
    "src/data/predeployed_contracts/ERC20Mintable/sierra.json.gz",
];
const PREDEPLOYED_CONTRACTS_SCRIPT: &str = "scripts/setup_predeployed_contracts.sh";

fn main() {
    // Re-run the build script when its validation logic changes, even if the
    // generated artifacts themselves stay untouched.
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../{PREDEPLOYED_CONTRACTS_SCRIPT}");

    for artifact in REQUIRED_ARTIFACTS {
        println!("cargo:rerun-if-changed={artifact}");
    }

    run_predeployed_contracts_script();

    let missing: Vec<_> = REQUIRED_ARTIFACTS
        .iter()
        .copied()
        .filter(|artifact| !Path::new(artifact).is_file())
        .collect();

    if !missing.is_empty() {
        let missing_files = missing
            .iter()
            .map(|artifact| format!("  - {artifact}"))
            .collect::<Vec<_>>()
            .join("\n");

        panic!(
            "Missing generated predeployed contract artifacts:\n{missing_files}\n\nRun `./scripts/setup_predeployed_contracts.sh` before building forge/cheatnet-dependent crates."
        );
    }
}

fn run_predeployed_contracts_script() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be set");
    let repo_root = Path::new(&manifest_dir)
        .parent()
        .and_then(Path::parent)
        .expect("cheatnet crate should live under the repository root");
    let script_path = repo_root.join(PREDEPLOYED_CONTRACTS_SCRIPT);

    let status = Command::new("bash")
        .arg(&script_path)
        .current_dir(repo_root)
        .status()
        .unwrap_or_else(|error| panic!("Failed to run {}: {error}", script_path.display()));

    assert!(
        status.success(),
        "Predeployed contracts setup script failed: {}",
        script_path.display()
    );
}
