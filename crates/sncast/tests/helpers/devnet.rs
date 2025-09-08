use crate::helpers::constants::{FORK_BLOCK_NUMBER, SEED, SEPOLIA_RPC_URL};
use crate::helpers::fixtures::{
    deploy_braavos_account, deploy_cairo_0_account, deploy_keystore_account,
    deploy_latest_oz_account, deploy_ready_account,
};
use ctor::{ctor, dtor};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::string::ToString;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use url::Url;

fn get_available_port() -> u16 {
    // 0 means "give me free port"
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind to address");
    let port = listener.local_addr().unwrap().port();
    port
}

fn is_list_mode() -> bool {
    std::env::args().any(|a| a == "--list" || a.starts_with("--list"))
}

#[expect(clippy::zombie_processes)]
#[cfg(test)]
#[ctor]
fn start_devnet() {
    use crate::helpers::constants::devnet_url;

    if is_list_mode() {
        return;
    }

    fn verify_devnet_availability(address: &str) -> bool {
        TcpStream::connect(address).is_ok()
    }

    let port = get_available_port();
    println!("XXX Starting devnet on port: {port}");

    let host = Url::parse(format!("http://127.0.0.1:{port}/rpc").as_str())
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
            &port.to_string(),
            "--seed",
            &SEED.to_string(),
            "--state-archive-capacity",
            "full",
            "--fork-network",
            SEPOLIA_RPC_URL,
            "--fork-block",
            &FORK_BLOCK_NUMBER.to_string(),
            "--initial-balance",
            "9999999999999999999999999999999",
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
    rt.block_on(deploy_ready_account());
    rt.block_on(deploy_braavos_account());
}

#[cfg(test)]
#[dtor]
fn stop_devnet() {
    use crate::helpers::constants::devnet_url;

    let port = Url::parse(&devnet_url())
        .unwrap()
        .port()
        .unwrap_or(80)
        .to_string();
    let pattern = format!("starknet-devnet.*{port}.*{SEED}");

    Command::new("pkill")
        .args(["-f", &pattern])
        .output()
        .expect("Failed to kill devnet processes");
}
