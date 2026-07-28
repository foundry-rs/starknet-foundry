mod direct;
mod docker;
mod flag_parsing;
use std::{future::Future, process::Command};
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
    #[error("Failed to parse devnet URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
}

pub async fn detect_devnet_url() -> Result<Url, DevnetDetectionError> {
    detect_devnet_from_processes(find_devnet_process_info(), is_devnet_url_reachable).await
}

#[must_use]
pub async fn is_devnet_running() -> bool {
    detect_devnet_from_processes(find_devnet_process_info(), is_devnet_url_reachable)
        .await
        .is_ok()
}

async fn detect_devnet_from_processes<F, Fut>(
    process_info: Result<ProcessInfo, DevnetDetectionError>,
    is_devnet_url_reachable: F,
) -> Result<Url, DevnetDetectionError>
where
    F: Fn(String, u16) -> Fut,
    Fut: Future<Output = bool>,
{
    match process_info {
        Ok(info) => {
            if is_devnet_url_reachable(info.host.clone(), info.port).await {
                Ok(Url::parse(&format!("http://{}:{}", info.host, info.port))?)
            } else {
                Err(DevnetDetectionError::ProcessNotReachable)
            }
        }
        Err(DevnetDetectionError::NoInstance | DevnetDetectionError::CommandFailed) => {
            // Fallback to default starknet-devnet URL if reachable
            if is_devnet_url_reachable(DEFAULT_DEVNET_HOST.to_string(), DEFAULT_DEVNET_PORT).await {
                Ok(Url::parse(&format!(
                    "http://{DEFAULT_DEVNET_HOST}:{DEFAULT_DEVNET_PORT}"
                ))?)
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

    extract_devnet_process_info(&ps_output)
}

fn extract_devnet_process_info(ps_output: &str) -> Result<ProcessInfo, DevnetDetectionError> {
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

async fn is_devnet_url_reachable(host: String, port: u16) -> bool {
    let url = format!("http://{host}:{port}");

    let provider = DevnetProvider::new(&url);
    provider.ensure_alive().await.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn detects_reachable_process() {
        let result = detect_devnet_from_processes(
            Ok(ProcessInfo {
                host: "127.0.0.1".to_string(),
                port: 5051,
            }),
            |_, _| async { true },
        )
        .await;

        assert_eq!(
            result.unwrap(),
            Url::parse("http://127.0.0.1:5051").unwrap()
        );
    }

    #[tokio::test]
    async fn returns_process_not_reachable_for_unreachable_process() {
        let result = detect_devnet_from_processes(
            Ok(ProcessInfo {
                host: "127.0.0.1".to_string(),
                port: 5051,
            }),
            |_, _| async { false },
        )
        .await;

        assert!(matches!(
            result.unwrap_err(),
            DevnetDetectionError::ProcessNotReachable
        ));
    }

    #[tokio::test]
    async fn falls_back_to_default_url_when_no_process_is_found() {
        let result =
            detect_devnet_from_processes(Err(DevnetDetectionError::NoInstance), |_, _| async {
                true
            })
            .await;

        assert_eq!(
            result.unwrap(),
            Url::parse("http://127.0.0.1:5050").unwrap()
        );
    }

    #[tokio::test]
    async fn falls_back_to_default_url_when_process_detection_command_fails() {
        let result =
            detect_devnet_from_processes(Err(DevnetDetectionError::CommandFailed), |_, _| async {
                true
            })
            .await;

        assert_eq!(
            result.unwrap(),
            Url::parse("http://127.0.0.1:5050").unwrap()
        );
    }

    #[tokio::test]
    async fn returns_no_instance_when_no_process_and_default_url_is_unreachable() {
        let result =
            detect_devnet_from_processes(Err(DevnetDetectionError::NoInstance), |_, _| async {
                false
            })
            .await;

        assert!(matches!(
            result.unwrap_err(),
            DevnetDetectionError::NoInstance
        ));
    }

    #[tokio::test]
    async fn preserves_multiple_instances_error() {
        let result = detect_devnet_from_processes(
            Err(DevnetDetectionError::MultipleInstances),
            |_, _| async { true },
        )
        .await;

        assert!(matches!(
            result.unwrap_err(),
            DevnetDetectionError::MultipleInstances
        ));
    }

    #[test]
    fn extracts_no_instance_from_empty_process_output() {
        let result = extract_devnet_process_info("");

        assert!(matches!(
            result.unwrap_err(),
            DevnetDetectionError::NoInstance
        ));
    }

    #[test]
    fn extracts_single_direct_process() {
        let ps_output = concat!(
            "runner 2685 0.0 0.1 100000 20000 ? Sl 13:00 0:00 ",
            "/home/runner/.asdf/shims/starknet-devnet --host 127.0.0.1 --port 5055"
        );

        let result = extract_devnet_process_info(ps_output).unwrap();

        assert_eq!(result.host, "127.0.0.1");
        assert_eq!(result.port, 5055);
    }

    #[test]
    fn extracts_multiple_instances_error() {
        let ps_output = "\
runner 2685 0.0 0.1 100000 20000 ? Sl 13:00 0:00 starknet-devnet --host 127.0.0.1 --port 5050
runner 2686 0.0 0.1 100000 20000 ? Sl 13:00 0:00 starknet-devnet --host 127.0.0.1 --port 5051";

        let result = extract_devnet_process_info(ps_output);

        assert!(matches!(
            result.unwrap_err(),
            DevnetDetectionError::MultipleInstances
        ));
    }

    #[tokio::test]
    async fn test_detect_devnet_url() {
        let result = detect_devnet_url().await;
        assert!(matches!(
            result,
            Ok(_) | Err(DevnetDetectionError::NoInstance | DevnetDetectionError::MultipleInstances)
        ));
    }
}
