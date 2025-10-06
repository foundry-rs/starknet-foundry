use std::process::Command;

#[derive(Debug, Clone)]
struct DevnetInfo {
    host: String,
    port: u16,
}

#[derive(Debug)]
enum FindDevnetError {
    None,
    Multiple,
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
        Ok(info) => Ok(format!("http://{}:{}", info.host, info.port)),
        Err(FindDevnetError::Multiple) => {
            Err("Multiple starknet-devnet instances found. Please use --url to specify which one to use.".to_string())
        }
        Err(FindDevnetError::None | FindDevnetError::CommandFailed) => {
            // Fallback to default starknet-
            if is_port_reachable("127.0.0.1", 5050) {
                Ok("http://127.0.0.1:5050".to_string())
            } else {
                Err("Could not detect running starknet-devnet instance. Please use --url instead.".to_string())
            }
        }
    }
}

fn find_devnet_process_info() -> Result<DevnetInfo, FindDevnetError> {
    let output = Command::new("ps")
        .args(["aux"])
        .output()
        .map_err(|_| FindDevnetError::CommandFailed)?;
    let ps_output = String::from_utf8_lossy(&output.stdout);

    let devnet_processes: Vec<DevnetInfo> = ps_output
        .lines()
        .filter(|line| line.contains("starknet-devnet"))
        .map(|line| {
            if line.contains("docker") || line.contains("podman") {
                extract_devnet_info_from_docker_line(line)
            } else {
                extract_devnet_info_from_cmdline(line)
            }
        })
        .collect();

    match devnet_processes.as_slice() {
        [] => Err(FindDevnetError::None),
        [single] => Ok(single.clone()),
        _ => Err(FindDevnetError::Multiple),
    }
}

fn extract_string_from_flag(cmdline: &str, flag: &str) -> Option<String> {
    if let Some(pos) = cmdline.find(flag) {
        let after_pattern = &cmdline[pos + flag.len()..];
        let value_str = after_pattern
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_start_matches('=')
            .trim_start_matches(':');

        if !value_str.is_empty() {
            return Some(value_str.to_string());
        }
    }
    None
}

fn extract_port_from_flag(cmdline: &str, flag: &str) -> Option<u16> {
    if let Some(port_str) = extract_string_from_flag(cmdline, flag)
        && let Ok(p) = port_str.parse::<u16>()
        && p > 1024
        && p < 65535
    {
        return Some(p);
    }

    None
}

fn extract_docker_port_mapping(cmdline: &str) -> Option<(String, u16)> {
    if let Some(pos) = cmdline.find("-p ") {
        let after_pattern = &cmdline[pos + 3..]; // "-p ".len() = 3
        let port_mapping = after_pattern.split_whitespace().next().unwrap_or("");

        let parts: Vec<&str> = port_mapping.split(':').collect();
        if parts.len() == 3
            && let Ok(external_port) = parts[1].parse::<u16>()
        {
            return Some((parts[0].to_string(), external_port));
        } else if parts.len() == 2
            && let Ok(external_port) = parts[0].parse::<u16>()
        {
            return Some(("127.0.0.1".to_string(), external_port));
        }
    }
    None
}

fn extract_devnet_info_from_docker_line(cmdline: &str) -> DevnetInfo {
    let mut port = None;
    let mut host = None;

    if let Some((docker_host, docker_port)) = extract_docker_port_mapping(cmdline) {
        host = Some(docker_host);
        port = Some(docker_port);
    }

    if port.is_none() {
        port = extract_port_from_flag(cmdline, "--port");
    }

    let final_host = host.unwrap_or_else(|| "127.0.0.1".to_string());
    let final_port = port.unwrap_or(5050);

    DevnetInfo {
        host: final_host,
        port: final_port,
    }
}

fn extract_devnet_info_from_cmdline(cmdline: &str) -> DevnetInfo {
    let mut port = extract_port_from_flag(cmdline, "--port");
    let mut host = extract_string_from_flag(cmdline, "--host");

    if port.is_none()
        && let Ok(port_env) = std::env::var("PORT")
        && let Ok(p) = port_env.parse::<u16>()
        && p > 1024
        && p < 65535
    {
        port = Some(p);
    }

    if host.is_none()
        && let Ok(host_env) = std::env::var("HOST")
        && !host_env.is_empty()
    {
        host = Some(host_env);
    }

    let final_port = port.unwrap_or(5050);
    let final_host = host.unwrap_or_else(|| "127.0.0.1".to_string());

    DevnetInfo {
        host: final_host,
        port: final_port,
    }
}

fn is_port_reachable(host: &str, port: u16) -> bool {
    let url = format!("http://{host}:{port}/is_alive");

    // TODO: Try to use a DevnetProvider::ensure_alive() from https://github.com/foundry-rs/starknet-foundry/pull/3760/
    std::process::Command::new("curl")
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

    // Those tests are marked to run serially to avoid interference from env vars
    #[test]
    fn test_devnet_parsing() {
        test_extract_devnet_info_from_cmdline();

        test_extract_devnet_info_from_docker_line();

        test_extract_devnet_info_with_both_envs();

        test_cmdline_args_override_env();

        test_detect_devnet_url();
    }

    fn test_extract_devnet_info_from_cmdline() {
        let cmdline1 = "starknet-devnet --port 5050 --host 127.0.0.1";
        let info1 = extract_devnet_info_from_cmdline(cmdline1);
        assert_eq!(info1.port, 5050);
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

        let cmdline2 = "docker run -p 8080:5050 shardlabs/starknet-devnet-rs";
        let info2 = extract_devnet_info_from_docker_line(cmdline2);
        assert_eq!(info2.port, 8080);
        assert_eq!(info2.host, "127.0.0.1");

        let cmdline3 = "docker run --network host shardlabs/starknet-devnet-rs --port 5055";
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
