use regex::Regex;
use std::process::Command;

const DEFAULT_DEVNET_HOST: &str = "127.0.0.1";
const DEFAULT_DEVNET_PORT: u16 = 5050;

#[derive(Debug, Clone)]
struct DevnetProcessInfo {
    host: String,
    port: u16,
}

#[derive(Debug)]
enum DevnetDetectionError {
    NoInstance,
    MultipleInstances,
    CommandFailed,
}

pub fn detect_devnet_url() -> Result<String, String> {
    detect_devnet_from_processes()
}

#[must_use]
pub fn is_devnet_running() -> bool {
    detect_devnet_from_processes().is_ok()
}

fn detect_devnet_from_processes() -> Result<String, String> {
    match find_devnet_process_info() {
        Ok(info) => {
            if is_port_reachable(&info.host, info.port) {
                Ok(format!("http://{}:{}", info.host, info.port))
            } else {
                Err(format!(
                    "Found starknet-devnet process, but could not reach it. Please use `--url <URL>` to specify the correct URL.",
                ))
            }
        }
        Err(DevnetDetectionError::MultipleInstances) => {
            Err("Multiple starknet-devnet instances found. Please use `--url <URL>` to specify which one to use.".to_string())
        }
        Err(DevnetDetectionError::NoInstance | DevnetDetectionError::CommandFailed) => {
            // Fallback to default starknet-devnet URL if reachable
            if is_port_reachable(DEFAULT_DEVNET_HOST, DEFAULT_DEVNET_PORT) {
                Ok(format!("http://{DEFAULT_DEVNET_HOST}:{DEFAULT_DEVNET_PORT}"))
            } else {
                Err(
                    "Could not detect running starknet-devnet instance. Please use `--url <URL>` instead or start devnet if it is not running."
                        .to_string(),
                )
            }
        }
    }
}

fn find_devnet_process_info() -> Result<DevnetProcessInfo, DevnetDetectionError> {
    let output = Command::new("sh")
        .args(["-c", "ps aux | grep starknet-devnet | grep -v grep"])
        .output()
        .map_err(|_| DevnetDetectionError::CommandFailed)?;
    let ps_output = String::from_utf8_lossy(&output.stdout);

    let devnet_processes: Vec<DevnetProcessInfo> = ps_output
        .lines()
        .map(|line| {
            if line.contains("docker") || line.contains("podman") {
                extract_devnet_info_from_docker_line(line)
            } else {
                extract_devnet_info_from_cmdline(line)
            }
        })
        .collect();

    match devnet_processes.as_slice() {
        [single] => Ok(single.clone()),
        [] => Err(DevnetDetectionError::NoInstance),
        _ => Err(DevnetDetectionError::MultipleInstances),
    }
}

fn extract_string_from_flag(cmdline: &str, flag: &str) -> Option<String> {
    let pattern = format!(r"{}\s*=?\s*(\S+)", regex::escape(flag));
    let re = Regex::new(&pattern).ok()?;

    re.captures(cmdline)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

fn extract_port_from_flag(cmdline: &str, flag: &str) -> Option<u16> {
    extract_string_from_flag(cmdline, flag).and_then(|port_str| port_str.parse().ok())
}

fn extract_docker_mapping(cmdline: &str) -> Option<(String, u16)> {
    let port_flags = ["-p", "--publish"];

    // Port mapping patterns:
    // - host:host_port:container_port (e.g., "127.0.0.1:5055:5050")
    // - host_port:container_port (e.g., "5090:5050")
    let re = Regex::new(r"^(?:([^:]+):)?(\d+):\d+$").ok()?;

    for flag in &port_flags {
        if let Some(port_mapping) = extract_string_from_flag(cmdline, flag)
            && let Some(caps) = re.captures(&port_mapping)
            && let Ok(host_port) = caps.get(2)?.as_str().parse::<u16>()
        {
            let host = caps.get(1).map_or_else(
                || DEFAULT_DEVNET_HOST.to_string(),
                |m| m.as_str().to_string(),
            );

            return Some((host, host_port));
        }
    }

    None
}

fn extract_devnet_info_from_docker_line(cmdline: &str) -> DevnetProcessInfo {
    let mut port = None;
    let mut host = None;

    if let Some((docker_host, docker_port)) = extract_docker_mapping(cmdline) {
        host = Some(docker_host);
        port = Some(docker_port);
    }

    if port.is_none()
        && extract_string_from_flag(cmdline, "--network").is_some_and(|network| network == "host")
    {
        port = extract_port_from_flag(cmdline, "--port");
    }

    let final_host = host.unwrap_or_else(|| DEFAULT_DEVNET_HOST.to_string());
    let final_port = port.unwrap_or(DEFAULT_DEVNET_PORT);

    DevnetProcessInfo {
        host: final_host,
        port: final_port,
    }
}

fn extract_devnet_info_from_cmdline(cmdline: &str) -> DevnetProcessInfo {
    let mut port = extract_port_from_flag(cmdline, "--port");
    let mut host = extract_string_from_flag(cmdline, "--host");

    if port.is_none() {
        port = std::env::var("PORT")
            .ok()
            .and_then(|port_env| port_env.parse().ok());
    

    if host.is_none()
        && let Ok(host_env) = std::env::var("HOST")
        && !host_env.is_empty()
    {
        host = Some(host_env);
    }

    let final_port = port.unwrap_or(DEFAULT_DEVNET_PORT);
    let final_host = host.unwrap_or_else(|| DEFAULT_DEVNET_HOST.to_string());

    DevnetProcessInfo {
        host: final_host,
        port: final_port,
    }
}

fn is_port_reachable(host: &str, port: u16) -> bool {
    let url = format!("http://{host}:{port}/is_alive");

    println!(
        "{:?}",
        Command::new("curl")
            .args(["-s", "-f", "--max-time", "1", &url])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    );

    Command::new("curl")
        .args(["-s", "-f", "--max-time", "1", &url])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    // These tests are marked to run serially to avoid interference from environment variables
    #[test]
    fn test_devnet_parsing() {
        test_extract_devnet_info_from_cmdline();

        test_extract_devnet_info_from_docker_line();

        test_extract_devnet_info_with_both_envs();

        test_cmdline_args_override_env();

        test_detect_devnet_url();
    }

    fn test_extract_devnet_info_from_cmdline() {
        let cmdline1 = "starknet-devnet --port 6000 --host 127.0.0.1";
        let info1 = extract_devnet_info_from_cmdline(cmdline1);
        assert_eq!(info1.port, 6000);
        assert_eq!(info1.host, "127.0.0.1");

        let cmdline2 = "/usr/bin/starknet-devnet --port=5000";
        let info2 = extract_devnet_info_from_cmdline(cmdline2);
        assert_eq!(info2.port, 5000);
        assert_eq!(info2.host, "127.0.0.1");

        let cmdline3 = "starknet-devnet --host 127.0.0.1";
        let info3 = extract_devnet_info_from_cmdline(cmdline3);
        assert_eq!(info3.port, 5050);
        assert_eq!(info3.host, "127.0.0.1");
    }

    fn test_extract_devnet_info_from_docker_line() {
        let cmdline1 = "docker run -p 127.0.0.1:5055:5050 shardlabs/starknet-devnet-rs";
        let info1 = extract_devnet_info_from_docker_line(cmdline1);
        assert_eq!(info1.port, 5055);
        assert_eq!(info1.host, "127.0.0.1");

        let cmdline2 = "docker run --publish     8080:5050 shardlabs/starknet-devnet-rs";
        let info2 = extract_devnet_info_from_docker_line(cmdline2);
        assert_eq!(info2.port, 8080);
        assert_eq!(info2.host, "127.0.0.1");

        let cmdline3 = "podman run --network host shardlabs/starknet-devnet-rs --port 5055";
        let info3 = extract_devnet_info_from_docker_line(cmdline3);
        assert_eq!(info3.port, 5055);
        assert_eq!(info3.host, "127.0.0.1");
    }

    fn test_extract_devnet_info_with_both_envs() {
        // SAFETY: Variables are only modified within this test and cleaned up afterwards
        unsafe {
            std::env::set_var("PORT", "9999");
            std::env::set_var("HOST", "9.9.9.9");
        }

        let cmdline = "starknet-devnet";
        let info = extract_devnet_info_from_cmdline(cmdline);
        assert_eq!(info.port, 9999);
        assert_eq!(info.host, "9.9.9.9");

        // SAFETY: Clean up environment variables to prevent interference
        unsafe {
            std::env::remove_var("PORT");
            std::env::remove_var("HOST");
        }
    }

    fn test_cmdline_args_override_env() {
        // SAFETY: Variables are only modified within this test and cleaned up afterwards
        unsafe {
            std::env::set_var("PORT", "3000");
            std::env::set_var("HOST", "7.7.7.7");
        }

        let cmdline = "starknet-devnet --port 9999 --host 192.168.1.1";
        let info = extract_devnet_info_from_cmdline(cmdline);
        assert_eq!(info.port, 9999);
        assert_eq!(info.host, "192.168.1.1");

        // SAFETY: Clean up environment variables to prevent interference
        unsafe {
            std::env::remove_var("PORT");
            std::env::remove_var("HOST");
        }
    }

    fn test_detect_devnet_url() {
        let child = spawn_devnet("5090");

        let result = detect_devnet_url().expect("Failed to detect devnet URL");
        assert_eq!(result, "http://127.0.0.1:5090");

        cleanup_process(child);
    }

    fn spawn_devnet(port: &str) -> std::process::Child {
        let mut child = Command::new("starknet-devnet")
            .args(["--port", port])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to spawn starknet-devnet process");

        let port_num: u16 = port.parse().expect("Invalid port number");
        let start_time = Instant::now();
        let timeout = Duration::from_secs(10);

        while start_time.elapsed() < timeout {
            if is_port_reachable("127.0.0.1", port_num) {
                return child;
            }
            thread::sleep(Duration::from_millis(500));
        }

        let _ = child.kill();
        let _ = child.wait();
        panic!("Devnet did not start in time on port {port}");
    }

    fn cleanup_process(mut child: std::process::Child) {
        child.kill().expect("Failed to kill devnet process");
        child.wait().expect("Failed to wait for devnet process");
    }
}
