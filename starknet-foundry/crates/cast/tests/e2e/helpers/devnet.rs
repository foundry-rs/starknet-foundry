use crate::helpers::constants::{SEED, URL};
use crate::helpers::fixtures::declare_deploy_simple_balance_contract;
use ctor::{ctor, dtor};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::string::ToString;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use url::Url;

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

    Command::new("starknet-devnet")
        .args([
            "--port",
            &port,
            "--seed",
            &SEED.to_string(),
            "--sierra-compiler-path",
            "tests/utils/cairo/bin/starknet-sierra-compile",
        ])
        .stdout(Stdio::null())
        .spawn()
        .expect("Failed to start devnet!");

    let now = Instant::now();
    let timeout = Duration::from_secs(10);

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
    rt.block_on(declare_deploy_simple_balance_contract());
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
}
