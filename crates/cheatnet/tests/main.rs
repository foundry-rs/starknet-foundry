mod cheatcodes;
pub(crate) mod common;
mod starknet;

// Build testing contracts before executing the tests
#[cfg(test)]
#[ctor::ctor]
fn init() {
    use camino::Utf8PathBuf;
    let contracts_path = Utf8PathBuf::from("tests").join("contracts");

    let output = std::process::Command::new("scarb")
        .current_dir(contracts_path)
        .arg("build")
        .output()
        .unwrap();
    if !output.status.success() {
        let stderr = String::from_utf8(output.stderr).expect("Decoding scarb stderr failed");
        let stdout = String::from_utf8(output.stdout).expect("Decoding scarb stdout failed");
        panic!("scarb build failed,\nstderr: \n{stderr}\nstdout: \n{stdout}",);
    }
}
