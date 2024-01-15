use super::fixtures::{convert_to_hex, get_address_from_keystore, mint_token};
use crate::helpers::constants::{DEVNET_ENV_FILE, DEVNET_OZ_CLASS_HASH, SEED, URL};
use crate::helpers::fixtures::{declare_contract, declare_deploy_contract, remove_devnet_env};
use ctor::{ctor, dtor};
use snapbox::cmd::cargo_bin;
use sncast::helpers::constants::KEYSTORE_PASSWORD_ENV_VAR;
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::string::ToString;
use std::time::{Duration, Instant};
use std::{env, fs};
use tokio::runtime::Runtime;
use url::Url;

#[cfg(test)]
#[ctor]
fn start_devnet() {
    fn verify_devnet_availability(address: &str) -> bool {
        TcpStream::connect(address).is_ok()
    }

    remove_devnet_env();
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

    Command::new("tests/utils/devnet/bin/starknet-devnet")
        .args(["--port", &port, "--seed", &SEED.to_string()])
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

    Command::new("tests/utils/build_contracts.sh")
        .spawn()
        .expect("Failed to compile contracts")
        .wait()
        .expect("Timed out compiling contracts");

    let rt = Runtime::new().expect("Could not instantiate Runtime");
    rt.block_on(declare_deploy_contract(
        "user1",
        "/map/target/dev/map_Map",
        "CAST_MAP",
    ));
    rt.block_on(declare_contract(
        "user4",
        "/constructor_with_params/target/dev/constructor_with_params_ConstructorWithParams",
        "CAST_WITH_CONSTRUCTOR",
    ));

    rt.block_on(deploy_keystore_account());

    dotenv::from_filename(DEVNET_ENV_FILE).unwrap();
}

async fn deploy_keystore_account() {
    let keystore_path = "tests/data/keystore/deployed_key.json";
    let account_path = "tests/data/keystore/deployed_account_copy.json";

    fs::copy("tests/data/keystore/deployed_account.json", account_path).unwrap();
    env::set_var(KEYSTORE_PASSWORD_ENV_VAR, "123");

    let address = get_address_from_keystore(keystore_path, account_path, KEYSTORE_PASSWORD_ENV_VAR);

    mint_token(
        &convert_to_hex(&address.to_string()),
        9_999_999_999_999_999_999,
    )
    .await;

    let args = vec![
        "--url",
        URL,
        "--keystore",
        keystore_path,
        "--account",
        account_path,
        "account",
        "deploy",
        "--max-fee",
        "99999999999999999",
        "--class-hash",
        DEVNET_OZ_CLASS_HASH,
    ];

    Command::new(cargo_bin!("sncast"))
        .args(args)
        .output()
        .unwrap();
}

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
    fs::remove_file("tests/data/keystore/deployed_account_copy.json").unwrap();
    remove_devnet_env();
}
