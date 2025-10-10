use regex::Regex;
use std::process::Command;

use crate::helpers::devnet::provider::DevnetProvider;

const DEFAULT_DEVNET_HOST: &str = "127.0.0.1";
const DEFAULT_DEVNET_PORT: u16 = 5050;

#[derive(Debug, Clone)]
struct DevnetProcessInfo {
    host: String,
    port: u16,
}

#[derive(Debug, thiserror::Error)]
enum DevnetDetectionError {
    #[error(
        "Could not detect running starknet-devnet instance. Please use `--url <URL>` instead or start devnet if it is not running."
    )]
    NoInstance,
    #[error(
        "Multiple starknet-devnet instances found. Please use `--url <URL>` to specify which one to use."
    )]
    MultipleInstances,
    #[error("Failed to execute process detection command.")]
    CommandFailed,
    #[error(
        "Found starknet-devnet process, but could not reach it. Please use `--url <URL>` to specify the correct URL."
    )]
    ProcessNotReachable,
}

pub async fn detect_devnet_url() -> Result<String, String> {
    detect_devnet_from_processes().await
}

#[must_use]
pub async fn is_devnet_running() -> bool {
    detect_devnet_from_processes().await.is_ok()
}

async fn detect_devnet_from_processes() -> Result<String, String> {
    match find_devnet_process_info() {
        Ok(info) => {
            if is_port_reachable(&info.host, info.port).await {
                Ok(format!("http://{}:{}", info.host, info.port))
            } else {
                Err(DevnetDetectionError::ProcessNotReachable.to_string())
            }
        }
        Err(DevnetDetectionError::MultipleInstances) => {
            Err(DevnetDetectionError::MultipleInstances.to_string())
        }
        Err(DevnetDetectionError::NoInstance | DevnetDetectionError::CommandFailed) => {
            // Fallback to default starknet-devnet URL if reachable
            if is_port_reachable(DEFAULT_DEVNET_HOST, DEFAULT_DEVNET_PORT).await {
                Ok(format!(
                    "http://{DEFAULT_DEVNET_HOST}:{DEFAULT_DEVNET_PORT}"
                ))
            } else {
                Err(DevnetDetectionError::NoInstance.to_string())
            }
        }
        Err(DevnetDetectionError::ProcessNotReachable) => {
            Err(DevnetDetectionError::ProcessNotReachable.to_string())
        }
    }
}

fn find_devnet_process_info() -> Result<DevnetProcessInfo, DevnetDetectionError> {
    let output = Command::new("sh")
        .args(["-c", "ps aux | grep starknet-devnet | grep -v grep"])
        .output()
        .map_err(|_| DevnetDetectionError::CommandFailed)?;
    let ps_output = String::from_utf8_lossy(&output.stdout);

    let devnet_processes: Result<Vec<DevnetProcessInfo>, DevnetDetectionError> = ps_output
        .lines()
        .map(|line| {
            if line.contains("docker") || line.contains("podman") {
                extract_devnet_info_from_docker_line(line)
            } else {
                extract_devnet_info_from_cmdline(line)
            }
        })
        .collect();

    let devnet_processes = devnet_processes?;

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

fn extract_devnet_info_from_docker_line(
    cmdline: &str,
) -> Result<DevnetProcessInfo, DevnetDetectionError> {
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

    // If port or host are still None, it means neither docker flags nor command line provided them (e.g., docker run shardlabs/starknet-devnet-rs)
    // which means we cannot connect to the process from outside the container
    let final_host = host.ok_or(DevnetDetectionError::ProcessNotReachable)?;
    let final_port = port.ok_or(DevnetDetectionError::ProcessNotReachable)?;

    Ok(DevnetProcessInfo {
        host: final_host,
        port: final_port,
    })
}

fn extract_devnet_info_from_cmdline(
    cmdline: &str,
) -> Result<DevnetProcessInfo, DevnetDetectionError> {
    let mut port = extract_port_from_flag(cmdline, "--port");
    let mut host = extract_string_from_flag(cmdline, "--host");

    if port.is_none()
        && let Ok(port_env) = std::env::var("PORT")
    {
        port = Some(
            port_env
                .parse()
                .map_err(|_| DevnetDetectionError::ProcessNotReachable)?,
        );
    }

    if host.is_none()
        && let Ok(host_env) = std::env::var("HOST")
        && !host_env.is_empty()
    {
        host = Some(host_env);
    }

    // If port or host are still None, it means neither command line nor env vars provided them, (e.g starknet-devnet --seed 0)
    let final_port = port.unwrap_or(DEFAULT_DEVNET_PORT);
    let final_host = host.unwrap_or_else(|| DEFAULT_DEVNET_HOST.to_string());

    Ok(DevnetProcessInfo {
        host: final_host,
        port: final_port,
    })
}

async fn is_port_reachable(host: &str, port: u16) -> bool {
    let url = format!("http://{host}:{port}");

    let provider = DevnetProvider::new(&url);
    provider.ensure_alive().await.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests are marked to run serially to avoid interference from environment variables
    #[test]
    fn test_devnet_parsing() {
        test_extract_devnet_info_from_cmdline();

        test_extract_devnet_info_from_docker_line();

        test_extract_devnet_info_with_both_envs();

        test_cmdline_args_override_env();

        test_invalid_env();
    }

    fn test_extract_devnet_info_from_cmdline() {
        let cmdline1 = "starknet-devnet --port 6000 --host 127.0.0.1";
        let info1 = extract_devnet_info_from_cmdline(cmdline1).unwrap();
        assert_eq!(info1.port, 6000);
        assert_eq!(info1.host, "127.0.0.1");

        let cmdline2 = "/usr/bin/starknet-devnet --port=5000";
        let info2 = extract_devnet_info_from_cmdline(cmdline2).unwrap();
        assert_eq!(info2.port, 5000);
        assert_eq!(info2.host, "127.0.0.1");

        let cmdline3 = "starknet-devnet --host 127.0.0.1";
        let info3 = extract_devnet_info_from_cmdline(cmdline3).unwrap();
        assert_eq!(info3.port, 5050);
        assert_eq!(info3.host, "127.0.0.1");
    }

    fn test_extract_devnet_info_from_docker_line() {
        let cmdline1 = "docker run -p 127.0.0.1:5055:5050 shardlabs/starknet-devnet-rs";
        let info1 = extract_devnet_info_from_docker_line(cmdline1).unwrap();
        assert_eq!(info1.port, 5055);
        assert_eq!(info1.host, "127.0.0.1");

        let cmdline2 = "docker run --publish     8080:5050 shardlabs/starknet-devnet-rs";
        let info2 = extract_devnet_info_from_docker_line(cmdline2).unwrap();
        assert_eq!(info2.port, 8080);
        assert_eq!(info2.host, "127.0.0.1");

        let cmdline3 = "podman run --network host shardlabs/starknet-devnet-rs --port 5055";
        let info3 = extract_devnet_info_from_docker_line(cmdline3).unwrap();
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
        let info = extract_devnet_info_from_cmdline(cmdline).unwrap();
        assert_eq!(info.port, 9999);
        assert_eq!(info.host, "9.9.9.9");

        // SAFETY: Clean up environment variables to prevent interference
        unsafe {
            std::env::remove_var("PORT");
            std::env::remove_var("HOST");
        }
    }

    fn test_invalid_env() {
        // SAFETY: Variables are only modified within this test and cleaned up afterwards
        unsafe {
            std::env::set_var("PORT", "asdf");
            std::env::set_var("HOST", "9.9.9.9");
        }
        let cmdline = "starknet-devnet";
        let result = extract_devnet_info_from_cmdline(cmdline);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DevnetDetectionError::ProcessNotReachable
        ));

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
        let info = extract_devnet_info_from_cmdline(cmdline).unwrap();
        assert_eq!(info.port, 9999);
        assert_eq!(info.host, "192.168.1.1");

        // SAFETY: Clean up environment variables to prevent interference
        unsafe {
            std::env::remove_var("PORT");
            std::env::remove_var("HOST");
        }
    }
}
