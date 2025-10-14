use crate::helpers::devnet::detection::flag_parsing::{
    extract_port_from_flag, extract_string_from_flag,
};
use crate::helpers::devnet::detection::{
    DEFAULT_DEVNET_HOST, DEFAULT_DEVNET_PORT, DevnetDetectionError, ProcessInfo,
};

pub fn extract_devnet_info_from_direct_run(
    cmdline: &str,
) -> Result<ProcessInfo, DevnetDetectionError> {
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

    let final_port = port.unwrap_or(DEFAULT_DEVNET_PORT);
    let final_host = host.unwrap_or_else(|| DEFAULT_DEVNET_HOST.to_string());

    Ok(ProcessInfo {
        host: final_host,
        port: final_port,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests are marked to run serially to avoid interference from environment variables
    #[test]
    fn test_direct_devnet_parsing() {
        test_extract_devnet_info_from_cmdline();
        test_extract_devnet_info_with_both_envs();
        test_invalid_env();
        test_cmdline_args_override_env();
        test_wrong_env_var();
    }

    fn test_extract_devnet_info_from_cmdline() {
        let cmdline1 = "starknet-devnet --port 6000 --host 127.0.0.1";
        let info1 = extract_devnet_info_from_direct_run(cmdline1).unwrap();
        assert_eq!(info1.port, 6000);
        assert_eq!(info1.host, "127.0.0.1");

        let cmdline2 = "/usr/bin/starknet-devnet --port=5000";
        let info2 = extract_devnet_info_from_direct_run(cmdline2).unwrap();
        assert_eq!(info2.port, 5000);
        assert_eq!(info2.host, "127.0.0.1");

        let cmdline3 = "starknet-devnet --host 127.0.0.1";
        let info3 = extract_devnet_info_from_direct_run(cmdline3).unwrap();
        assert_eq!(info3.port, 5050);
        assert_eq!(info3.host, "127.0.0.1");
    }

    fn test_extract_devnet_info_with_both_envs() {
        // SAFETY: Variables are only modified within this test and cleaned up afterwards
        unsafe {
            std::env::set_var("PORT", "9999");
            std::env::set_var("HOST", "9.9.9.9");
        }

        let cmdline = "starknet-devnet";
        let info = extract_devnet_info_from_direct_run(cmdline).unwrap();
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
        let result = extract_devnet_info_from_direct_run(cmdline);
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
        let info = extract_devnet_info_from_direct_run(cmdline).unwrap();
        assert_eq!(info.port, 9999);
        assert_eq!(info.host, "192.168.1.1");

        // SAFETY: Clean up environment variables to prevent interference
        unsafe {
            std::env::remove_var("PORT");
            std::env::remove_var("HOST");
        }
    }

    fn test_wrong_env_var() {
        // SAFETY: Variables are only modified within this test and cleaned up afterwards
        unsafe {
            std::env::set_var("PORT", "asdf");
        }

        // Empty HOST env var should be ignored and defaults should be used
        let cmdline = "starknet-devnet";
        let result = extract_devnet_info_from_direct_run(cmdline);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DevnetDetectionError::ProcessNotReachable
        ));

        // SAFETY: Clean up environment variables to prevent interference
        unsafe {
            std::env::remove_var("PORT");
        }
    }
}
