use std::{env, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let artifacts = [
        "src/data/predeployed_contracts/ERC20Lockable/casm.json.gz",
        "src/data/predeployed_contracts/ERC20Lockable/sierra.json.gz",
        "src/data/predeployed_contracts/ERC20Mintable/casm.json.gz",
        "src/data/predeployed_contracts/ERC20Mintable/sierra.json.gz",
    ];

    for relative_path in artifacts {
        let path = manifest_dir.join(relative_path);
        println!("cargo:rerun-if-changed={}", path.display());

        if !path.is_file() {
            panic!(
                "missing predeployed contract artifact at {}\n\n
run ./scripts/setup_predeployed_contracts.sh before building\n",
                path.display()
            );
        }
    }

    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir
            .join("../../scripts/setup_predeployed_contracts.sh")
            .display()
    );
}
