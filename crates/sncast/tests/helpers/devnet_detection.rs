use sncast::helpers::devnet::detection::{DevnetDetectionError, detect_devnet_url};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use crate::helpers::constants::URL;

// These tests are marked to run serially to avoid interference from second devnet instance

#[tokio::test]
async fn test_devnet_detection() {
    test_detect_devnet_url().await;
    test_multiple_devnet_instances_error().await;
}

async fn test_detect_devnet_url() {
    let result = detect_devnet_url()
        .await
        .expect("Failed to detect devnet URL");

    assert_eq!(result, URL.replace("/rpc", ""));
}

async fn test_multiple_devnet_instances_error() {
    let mut devnet1 = start_devnet_instance(5051, 1234);

    wait_for_devnet("127.0.0.1:5051", Duration::from_secs(10));

    let result = detect_devnet_url().await;

    let _ = devnet1.kill();
    let _ = devnet1.wait();

    assert!(matches!(
        result,
        Err(DevnetDetectionError::MultipleInstances)
    ));
}

fn start_devnet_instance(port: u16, seed: u32) -> Child {
    Command::new("starknet-devnet")
        .args([
            "--port",
            &port.to_string(),
            "--seed",
            &seed.to_string(),
            "--accounts",
            "1",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("Failed to start devnet instance")
}

fn wait_for_devnet(address: &str, timeout: Duration) {
    let now = Instant::now();
    loop {
        if TcpStream::connect(address).is_ok() {
            break;
        } else if now.elapsed() >= timeout {
            panic!("Timed out waiting for devnet at {address}");
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}
