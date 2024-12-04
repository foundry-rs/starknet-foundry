use crate::helpers::constants::{FORK_BLOCK_NUMBER, SEED, SEPOLIA_RPC_URL, URL};
use crate::helpers::fixtures::{
    deploy_argent_account, deploy_braavos_account, deploy_cairo_0_account, deploy_keystore_account,
    deploy_latest_oz_account,
};
use camino::Utf8PathBuf;
use ctor::{ctor, dtor};
use regex::Regex;
use shared::test_utils::output_assert::AsOutput;
use std::collections::HashMap;
use std::fs;
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::string::ToString;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::runtime::Runtime;
use url::Url;

use super::runner::runner;

pub struct Contract {
    pub class_hash: String,
    pub contract_address: String,
}

#[allow(clippy::zombie_processes)]
#[cfg(test)]
#[ctor]
fn start_devnet() {
    fn verify_devnet_availability(address: &str) -> bool {
        TcpStream::connect(address).is_ok()
    }

    let port = Url::parse(URL).unwrap().port().unwrap_or(80).to_string();
    let host = Url::parse(URL)
        .unwrap()
        .host()
        .expect("Can't parse devnet URL!")
        .to_string();

    loop {
        if verify_devnet_availability(&format!("{host}:{port}")) {
            stop_devnet();
        } else {
            break;
        }
    }

    Command::new("tests/utils/devnet/starknet-devnet")
        .args([
            "--port",
            &port,
            "--seed",
            &SEED.to_string(),
            "--state-archive-capacity",
            "full",
            "--fork-network",
            SEPOLIA_RPC_URL,
            "--fork-block",
            &FORK_BLOCK_NUMBER.to_string(),
            "--initial-balance",
            "9999999999999999999",
            "--accounts",
            "20",
        ])
        .stdout(Stdio::null())
        .spawn()
        .expect("Failed to start devnet!");

    let now = Instant::now();
    let timeout = Duration::from_secs(30);

    loop {
        if verify_devnet_availability(&format!("{host}:{port}")) {
            break;
        } else if now.elapsed() >= timeout {
            eprintln!("Timed out while waiting for devnet!");
            std::process::exit(1);
        }
    }

    let rt = Runtime::new().expect("Could not instantiate Runtime");

    rt.block_on(deploy_keystore_account());
    rt.block_on(deploy_cairo_0_account());
    rt.block_on(deploy_latest_oz_account());
    rt.block_on(deploy_argent_account());
    rt.block_on(deploy_braavos_account());
}

#[allow(clippy::zombie_processes)]
#[cfg(test)]
#[dtor]
fn stop_devnet() {
    let port = Url::parse(URL).unwrap().port().unwrap_or(80).to_string();
    Command::new("pkill")
        .args([
            "-f",
            &format!("starknet-devnet.*{}.*{}", &port, &SEED.to_string())[..],
        ])
        .spawn()
        .expect("Failed to kill devnet processes");
}

fn declare_and_deploy_contract(
    contract_name: &str,
    accounts_file: &str,
    temp: &TempDir,
) -> Contract {
    let args = vec![
        "--accounts-file",
        accounts_file,
        "--account",
        "my_account",
        "declare",
        "--url",
        URL,
        "--contract-name",
        contract_name,
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "strk",
    ];

    let snapbox = runner(&args).current_dir(temp.path());
    let output = snapbox.assert().success();
    let re_class_hash = Regex::new(r"class_hash:\s+(0x[a-fA-F0-9]+)").unwrap();

    let class_hash = re_class_hash
        .captures(output.as_stdout())
        .and_then(|captures| captures.get(1))
        .map(|match_| match_.as_str())
        .expect("class_hash not found in the output");

    let args = vec![
        "--accounts-file",
        accounts_file,
        "--account",
        "my_account",
        "deploy",
        "--url",
        URL,
        "--class-hash",
        class_hash,
        "--max-fee",
        "99999999999999999",
        "--fee-token",
        "strk",
    ];

    let re_contract_address = Regex::new(r"contract_address:\s+(0x[a-fA-F0-9]+)").unwrap();

    let snapbox = runner(&args).current_dir(temp.path());
    let output = snapbox.assert().success();

    let contract_address = re_contract_address
        .captures(output.as_stdout())
        .and_then(|captures| captures.get(1))
        .map(|match_| match_.as_str())
        .expect("contract_address not found in the output");

    Contract {
        class_hash: class_hash.to_string(),
        contract_address: contract_address.to_string(),
    }
}

pub fn prepare_accounts_file(temp: &TempDir) -> Utf8PathBuf {
    // Account from predeployed accounts in starknet-devnet-rs
    let accounts = r#"
    {
        "alpha-sepolia": {
            "my_account": {
            "address": "0x6f4621e7ad43707b3f69f9df49425c3d94fdc5ab2e444bfa0e7e4edeff7992d",
            "deployed": true,
            "private_key": "0x0000000000000000000000000000000056c12e097e49ea382ca8eadec0839401",
            "public_key": "0x048234b9bc6c1e749f4b908d310d8c53dae6564110b05ccf79016dca8ce7dfac",
            "type": "open_zeppelin"
            }
        }
    }
    "#;

    let accounts_path = temp.path().join("accounts.json");
    fs::write(&accounts_path, accounts).expect("Failed to write accounts.json");

    Utf8PathBuf::from_path_buf(accounts_path).expect("Invalid UTF-8 path")
}

pub fn setup_contracts_map(
    tempdir: &TempDir,
    account_json_path: &Utf8PathBuf,
) -> HashMap<String, Contract> {
    let mut contracts: HashMap<String, Contract> = HashMap::new();
    let contract_names = [
        "HelloSncast",
        "DataTransformerContract",
        "ConstructorContract",
    ];

    for contract_name in &contract_names {
        let contract =
            declare_and_deploy_contract(contract_name, account_json_path.as_str(), tempdir);
        contracts.insert((*contract_name).to_string(), contract);
    }

    contracts
}
