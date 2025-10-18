use regex::Regex;

use crate::helpers::devnet::detection::flag_parsing::{
    extract_port_from_flag, extract_string_from_flag,
};
use crate::helpers::devnet::detection::{DEFAULT_DEVNET_HOST, DevnetDetectionError, ProcessInfo};

pub fn extract_docker_mapping(cmdline: &str) -> Option<ProcessInfo> {
    let port_flags = ["-p", "--publish"];

    // Port mapping patterns:
    // - host:host_port:container_port (e.g., "127.0.0.1:5055:5050")
    // - host_port:container_port (e.g., "5090:5050")
    let re = Regex::new(r"^(?:([^:]+):)?(\d+):\d+$").ok()?;

    for flag in &port_flags {
        if let Some(port_mapping) = extract_string_from_flag(cmdline, flag)
            && let Some(caps) = re.captures(&port_mapping)
            && let Ok(port) = caps.get(2)?.as_str().parse::<u16>()
        {
            let host = caps.get(1).map_or_else(
                || DEFAULT_DEVNET_HOST.to_string(),
                |m| m.as_str().to_string(),
            );

            return Some(ProcessInfo { host, port });
        }
    }

    None
}

pub fn extract_devnet_info_from_docker_run(
    cmdline: &str,
) -> Result<ProcessInfo, DevnetDetectionError> {
    if let Some(docker_info) = extract_docker_mapping(cmdline) {
        return Ok(docker_info);
    }

    if extract_string_from_flag(cmdline, "--network").is_some_and(|network| network == "host")
        && let Some(port) = extract_port_from_flag(cmdline, "--port")
    {
        return Ok(ProcessInfo {
            host: DEFAULT_DEVNET_HOST.to_string(),
            port,
        });
    }

    // If connection info was not provided (e.g., docker run shardlabs/starknet-devnet-rs),
    // we cannot connect to the process from outside the container
    Err(DevnetDetectionError::ProcessNotReachable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_devnet_info_from_docker_line() {
        let cmdline1 = "docker run -p 127.0.0.1:5055:5050 shardlabs/starknet-devnet-rs";
        let info1 = extract_devnet_info_from_docker_run(cmdline1).unwrap();
        assert_eq!(info1.port, 5055);
        assert_eq!(info1.host, "127.0.0.1");

        let cmdline2 = "docker run --publish     8080:5050 shardlabs/starknet-devnet-rs";
        let info2 = extract_devnet_info_from_docker_run(cmdline2).unwrap();
        assert_eq!(info2.port, 8080);
        assert_eq!(info2.host, "127.0.0.1");

        let cmdline3 = "podman run --network host shardlabs/starknet-devnet-rs --port 5055";
        let info3 = extract_devnet_info_from_docker_run(cmdline3).unwrap();
        assert_eq!(info3.port, 5055);
        assert_eq!(info3.host, "127.0.0.1");
    }

    #[test]
    fn test_extract_docker_mapping_helper() {
        let line = "docker run -p 127.0.0.1:5055:5050 shardlabs/starknet-devnet-rs";
        let info = extract_docker_mapping(line).expect("mapping should parse");
        assert_eq!(info.host, "127.0.0.1");
        assert_eq!(info.port, 5055);

        let line2 = "docker run --publish 8080:5050 shardlabs/starknet-devnet-rs";
        let info2 = extract_docker_mapping(line2).expect("mapping should parse");
        assert_eq!(info2.host, "127.0.0.1");
        assert_eq!(info2.port, 8080);
    }

    #[test]
    fn test_docker_without_port_mapping() {
        let cmdline = "docker run shardlabs/starknet-devnet-rs";
        let result = extract_devnet_info_from_docker_run(cmdline);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DevnetDetectionError::ProcessNotReachable
        ));
    }

    #[test]
    fn test_docker_with_invalid_port_mapping() {
        let cmdline = "docker run -p invalid shardlabs/starknet-devnet-rs";
        let result = extract_devnet_info_from_docker_run(cmdline);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DevnetDetectionError::ProcessNotReachable
        ));
    }

    #[test]
    fn test_docker_network_host_without_port() {
        let cmdline = "docker run --network host shardlabs/starknet-devnet-rs";
        let result = extract_devnet_info_from_docker_run(cmdline);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DevnetDetectionError::ProcessNotReachable
        ));
    }
}
