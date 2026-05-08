use std::path::Path;

const REQUIRED_ARTIFACTS: &[&str] = &[
    "src/data/predeployed_contracts/ERC20Lockable/casm.json.gz",
    "src/data/predeployed_contracts/ERC20Lockable/sierra.json.gz",
    "src/data/predeployed_contracts/ERC20Mintable/casm.json.gz",
    "src/data/predeployed_contracts/ERC20Mintable/sierra.json.gz",
];

fn main() {
    // Re-run the build script when its validation logic changes, even if the
    // generated artifacts themselves stay untouched.
    println!("cargo:rerun-if-changed=build.rs");

    for artifact in REQUIRED_ARTIFACTS {
        println!("cargo:rerun-if-changed={artifact}");
    }

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
