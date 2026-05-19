use flate2::{Compression, write::GzEncoder};
use std::{
    env, fs,
    io::Write,
    path::{Path, PathBuf},
};

const REQUIRED_ARTIFACTS: &[(&str, &str)] = &[
    ("ERC20Lockable", "casm.json"),
    ("ERC20Lockable", "sierra.json"),
    ("ERC20Mintable", "casm.json"),
    ("ERC20Mintable", "sierra.json"),
];
const PREDEPLOYED_CONTRACTS_SOURCE_DIR: &str = "src/data/predeployed_contracts";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR should be set"));

    let missing: Vec<_> = REQUIRED_ARTIFACTS
        .iter()
        .filter_map(|(contract_dir, artifact_name)| {
            let source_path = source_artifact_path(contract_dir, artifact_name);
            println!("cargo:rerun-if-changed={}", source_path.display());

            (!source_path.is_file()).then_some(source_path)
        })
        .collect();

    if !missing.is_empty() {
        let missing_files = missing
            .iter()
            .map(|artifact| format!("  - {}", artifact.display()))
            .collect::<Vec<_>>()
            .join("\n");

        panic!(
            "Missing predeployed contract artifacts:\n{missing_files}\n\nRun `./scripts/setup_predeployed_contracts.sh` and commit the generated files."
        );
    }

    for (contract_dir, artifact_name) in REQUIRED_ARTIFACTS {
        let source_path = source_artifact_path(contract_dir, artifact_name);
        let output_path = out_dir
            .join("predeployed_contracts")
            .join(contract_dir)
            .join(format!("{artifact_name}.gz"));

        gzip_artifact(&source_path, &output_path);
    }
}

fn source_artifact_path(contract_dir: &str, artifact_name: &str) -> PathBuf {
    Path::new(PREDEPLOYED_CONTRACTS_SOURCE_DIR)
        .join(contract_dir)
        .join(artifact_name)
}

fn gzip_artifact(source_path: &Path, output_path: &Path) {
    let artifact = fs::read(source_path)
        .unwrap_or_else(|error| panic!("Failed to read {}: {error}", source_path.display()));
    let parent = output_path.parent().unwrap_or_else(|| {
        panic!(
            "Failed to resolve output directory for {}",
            output_path.display()
        )
    });

    fs::create_dir_all(parent)
        .unwrap_or_else(|error| panic!("Failed to create {}: {error}", parent.display()));

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(&artifact).unwrap_or_else(|error| {
        panic!(
            "Failed to gzip artifact from {}: {error}",
            source_path.display()
        )
    });
    let gzipped = encoder.finish().unwrap_or_else(|error| {
        panic!(
            "Failed to finish gzip artifact from {}: {error}",
            source_path.display()
        )
    });

    fs::write(output_path, gzipped)
        .unwrap_or_else(|error| panic!("Failed to write {}: {error}", output_path.display()));
}
