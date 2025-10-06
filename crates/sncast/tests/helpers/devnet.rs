use crate::helpers::constants::{
    DEVNET_ACCOUNTS_NUMBER, DEVNET_FORK_BLOCK_NUMBER, DEVNET_SEED, SEPOLIA_RPC_URL, URL,
};
use crate::helpers::fixtures::{
    deploy_braavos_account, deploy_cairo_0_account, deploy_keystore_account,
    deploy_latest_oz_account, deploy_ready_account,
};
use ctor::{ctor, dtor};
use std::net::TcpStream;
use std::process::{Command, Stdio};
use std::string::ToString;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use url::Url;

#[expect(clippy::zombie_processes)]
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

    Command::new("/Users/franciszekjob/Downloads/starknet-devnet-0.6.0")
        .args([
            "--port",
            &port,
            "--seed",
            &DEVNET_SEED.to_string(),
            "--state-archive-capacity",
            "full",
            "--fork-network",
            SEPOLIA_RPC_URL,
            "--fork-block",
            &DEVNET_FORK_BLOCK_NUMBER.to_string(),
            "--initial-balance",
            "9999999999999999999999999999999",
            "--accounts",
            &DEVNET_ACCOUNTS_NUMBER.to_string(),
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
    rt.block_on(deploy_ready_account());
    rt.block_on(deploy_braavos_account());
}

#[cfg(test)]
#[dtor]
fn stop_devnet() {
    let port = Url::parse(URL).unwrap().port().unwrap_or(80).to_string();
    let pattern = format!("starknet-devnet.*{port}.*{DEVNET_SEED}");

    Command::new("pkill")
        .args(["-f", &pattern])
        .output()
        .expect("Failed to kill devnet processes");
}
