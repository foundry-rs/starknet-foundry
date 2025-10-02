use std::net::TcpStream;
use std::time::Duration;

/// Detects devnet by scanning running processes for starknet-devnet and extracting the port
#[must_use]
pub fn detect_devnet_url() -> String {
    detect_devnet_from_processes().unwrap_or_else(|| "http://localhost:5050".to_string())
}

#[must_use]
pub fn is_devnet_running() -> bool {
    detect_devnet_from_processes().is_some()
}

/// Detects devnet by scanning running processes for starknet-devnet and extracting the port
fn detect_devnet_from_processes() -> Option<String> {
    if let Some(port) = find_devnet_process_port() {
        return Some(format!("http://localhost:{port}"));
    }

    let common_ports = [5050, 5000, 8545, 3000, 8000];
    for &port in &common_ports {
        if is_port_reachable("localhost", port) {
            return Some(format!("http://localhost:{port}"));
        }
    }

    None
}

fn find_devnet_process_port() -> Option<u16> {
    use std::process::Command;

    let output = Command::new("ps").args(["aux"]).output().ok()?;
    let ps_output = String::from_utf8_lossy(&output.stdout);

    for line in ps_output.lines() {
        if line.contains("starknet-devnet") {
            // First try to extract port from command line arguments (faster)
            if let Some(port) = extract_port_from_cmdline(line) {
                return Some(port);
            }

            // If that fails, try to get port from PID
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                if let Ok(pid) = parts[1].parse::<u32>() {
                    if let Some(port) = get_port_from_pid(pid) {
                        return Some(port);
                    }
                }
            }
        }
    }
    None
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn get_port_from_pid(pid: u32) -> Option<u16> {
    if let Some(port) = try_lsof_for_port(pid) {
        return Some(port);
    }

    #[cfg(target_os = "linux")]
    {
        try_linux_nettools_for_port(pid)
    }

    None
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn try_lsof_for_port(pid: u32) -> Option<u16> {
    use std::process::Command;

    let output = Command::new("lsof")
        .args(["-P", "-p", &pid.to_string(), "-i"])
        .output()
        .ok()?;

    let lsof_output = String::from_utf8_lossy(&output.stdout);

    for line in lsof_output.lines() {
        if line.contains("TCP") && line.contains("LISTEN") {
            if let Some(port_part) = line.split_whitespace().last() {
                if let Some(port_str) = port_part.split(':').next_back() {
                    if let Ok(port) = port_str.parse::<u16>() {
                        return Some(port);
                    }
                }
            }
        }
    }
    None
}

#[cfg(target_os = "linux")]
fn try_linux_nettools_for_port(pid: u32) -> Option<u16> {
    use std::process::Command;

    let output = Command::new("ss")
        .args(&["-tlnp"])
        .output()
        .or_else(|_| Command::new("netstat").args(&["-tlnp"]).output())
        .ok()?;

    let net_output = String::from_utf8_lossy(&output.stdout);

    for line in net_output.lines() {
        if line.contains(&pid.to_string()) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 3 {
                if let Some(port_str) = parts[3].split(':').last() {
                    if let Ok(port) = port_str.parse::<u16>() {
                        return Some(port);
                    }
                }
            }
        }
    }
    None
}

fn extract_port_from_cmdline(cmdline: &str) -> Option<u16> {
    let patterns = ["--port", "--host", ":", "localhost:"];

    for pattern in &patterns {
        if let Some(pos) = cmdline.find(pattern) {
            let after_pattern = &cmdline[pos + pattern.len()..];
            let port_str = after_pattern
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_start_matches('=')
                .trim_start_matches(':');

            if let Ok(port) = port_str.parse::<u16>() {
                if port > 1000 && port < 65535 {
                    return Some(port);
                }
            }
        }
    }

    for word in cmdline.split_whitespace() {
        if let Ok(port) = word.parse::<u16>() {
            return Some(port);
        }
    }

    None
}

fn is_port_reachable(host: &str, port: u16) -> bool {
    if let Ok(addr) = format!("{host}:{port}").parse() {
        TcpStream::connect_timeout(&addr, Duration::from_millis(50)).is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_devnet_process_detection() {
        let cmdline1 = "starknet-devnet --port 5050 --host localhost";
        assert_eq!(extract_port_from_cmdline(cmdline1), Some(5050));

        let cmdline3 = "/usr/bin/starknet-devnet --port=5000";
        assert_eq!(extract_port_from_cmdline(cmdline3), Some(5000));

        // Test devnet URL generation
        let devnet_url = detect_devnet_url();
        assert!(devnet_url.starts_with("http://localhost:"));

        let _reachable = is_port_reachable("localhost", 5050);
    }
}
