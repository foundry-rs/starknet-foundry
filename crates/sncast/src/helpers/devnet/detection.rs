mod direct;
mod docker;
mod flag_parsing;
use std::process::Command;
use url::Url;

use crate::helpers::devnet::provider::DevnetProvider;

pub(super) const DEFAULT_DEVNET_HOST: &str = "127.0.0.1";
pub(super) const DEFAULT_DEVNET_PORT: u16 = 5050;

#[derive(Debug, Clone)]
pub(super) struct ProcessInfo {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, thiserror::Error)]
pub enum DevnetDetectionError {
    #[error(
        "Could not detect running starknet-devnet instance. Please use `--url <URL>` instead or start devnet."
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
    #[error("Failed to parse devnet URL.")]
    InvalidUrl {
        #[source]
        source: url::ParseError,
    },
}

pub async fn detect_devnet_url() -> Result<Url, DevnetDetectionError> {
    detect_devnet_from_processes().await
}

#[must_use]
pub async fn is_devnet_running() -> bool {
    detect_devnet_from_processes().await.is_ok()
}

async fn detect_devnet_from_processes() -> Result<Url, DevnetDetectionError> {
    match find_devnet_process_info() {
        Ok(info) => {
            if is_devnet_url_reachable(&info.host, info.port).await {
                Ok(Url::parse(&format!("http://{}:{}", info.host, info.port))
                    .map_err(|e| DevnetDetectionError::InvalidUrl { source: e })?)
            } else {
                Err(DevnetDetectionError::ProcessNotReachable)
            }
        }
        Err(DevnetDetectionError::NoInstance | DevnetDetectionError::CommandFailed) => {
            // Fallback to default starknet-devnet URL if reachable
            if is_devnet_url_reachable(DEFAULT_DEVNET_HOST, DEFAULT_DEVNET_PORT).await {
                Ok(Url::parse(&format!(
                    "http://{DEFAULT_DEVNET_HOST}:{DEFAULT_DEVNET_PORT}"
                ))
                .map_err(|e| DevnetDetectionError::InvalidUrl { source: e })?)
            } else {
                Err(DevnetDetectionError::NoInstance)
            }
        }
        Err(e) => Err(e),
    }
}

fn find_devnet_process_info() -> Result<ProcessInfo, DevnetDetectionError> {
    let output = Command::new("sh")
        .args(["-c", "ps aux | grep starknet-devnet | grep -v grep"])
        .output()
        .map_err(|_| DevnetDetectionError::CommandFailed)?;
    let ps_output = String::from_utf8_lossy(&output.stdout);

    let devnet_processes: Result<Vec<ProcessInfo>, DevnetDetectionError> = ps_output
        .lines()
        .map(|line| {
            if line.contains("docker") || line.contains("podman") {
                docker::extract_devnet_info_from_docker_run(line)
            } else {
                direct::extract_devnet_info_from_direct_run(line)
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

async fn is_devnet_url_reachable(host: &str, port: u16) -> bool {
    let url = format!("http://{host}:{port}");

    let provider = DevnetProvider::new(&url);
    provider.ensure_alive().await.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_detect_devnet_url() {
        let result = detect_devnet_url().await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DevnetDetectionError::NoInstance
        ));
    }
}
