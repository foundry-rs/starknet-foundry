use anyhow::{anyhow, bail, Context, Error, Result};
use camino::{Utf8Path, Utf8PathBuf};
use scarb::ops::find_manifest_path;
use scarb_metadata;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::env::current_dir;
use std::process::Command;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ScarbConfig {
    pub rpc_url: String,
    pub network: String,
    pub account: String,
}

fn get_tool_property(
    tool: &Option<BTreeMap<String, Value>>,
    profile: Option<String>,
    property: &str,
) -> Result<String, Error> {
    let profiled = tool.as_ref().and_then(|t| t.get("protostar"));

    match profile {
        Some(ref p) => profiled
            .and_then(|t| t.get(p))
            .and_then(|t| t.get(property))
            .and_then(|t| t.as_str())
            .map(String::from)
            .ok_or(anyhow!("Profile or property not found in Scarb.toml: {p}, {property}")),
        None => profiled
            .and_then(|t| t.get(property))
            .and_then(|t| t.as_str())
            .map(String::from)
            .ok_or(anyhow!("Property not found in tool: {property}")),
    }
}

pub fn parse_scarb_config(profile: Option<String>, path: Option<Utf8PathBuf>) -> Result<ScarbConfig> {
    let path = find_manifest_path(path.as_ref().map(|buf| buf.as_path())).expect("Failed to obtain Scarb.toml file path");
    let metadata = scarb_metadata::MetadataCommand::new()
        .inherit_stderr()
        .manifest_path(&path)
        .no_deps()
        .exec()
        .context(
            "Failed to read Scarb.toml manifest file, not found in current nor parent directories",
        )?;

    let package = &metadata.packages[0].manifest_metadata.tool;

    let url = get_tool_property(package, profile.clone(), "rpc_url")?;
    let network = get_tool_property(package, profile.clone(), "network")?;
    let account = get_tool_property(package, profile.clone(), "account")?;

    Ok(ScarbConfig {
        rpc_url: url,
        network: network,
        account: account,
    })
}

#[cfg(test)]
mod tests {
    use crate::helpers::scarb_utils::{parse_scarb_config, ScarbConfig};
    use camino::{Utf8Path, Utf8PathBuf};
    use std::env;

    #[test]
    fn test_parse_scarb_config_happy_case_with_profile() {
        let config = parse_scarb_config(
            Some(String::from("myprofile")),
            Some(Utf8PathBuf::from("tests/data/contracts/balance/Scarb.toml")),
        )
        .unwrap();

        assert_eq!(config.account, String::from("user1"));
        assert_eq!(config.network, String::from("testnet"));
        assert_eq!(
            config.rpc_url,
            String::from("http://127.0.0.1:5055/rpc")
        );
    }

    #[test]
    fn test_parse_scarb_config_happy_case_without_profile() {
        let config = parse_scarb_config(
            None,
            Some(Utf8PathBuf::from("tests/data/contracts/map/Scarb.toml")),
        )
        .unwrap();
        assert_eq!(config.account, String::from("user2"));
        assert_eq!(config.network, String::from("testnet"));
        assert_eq!(
            config.rpc_url,
            String::from("http://127.0.0.1:5055/rpc")
        );
    }

    #[test]
    fn test_parse_scarb_config_not_found() {
        let config = parse_scarb_config(None, None).unwrap_err();
        assert!(config.to_string().contains(
            "Failed to read Scarb.toml manifest file, not found in current nor parent directories"
        ));
    }

    #[test]
    fn test_parse_scarb_config_not_in_file() {
        let config = parse_scarb_config(
            None,
            Some(Utf8PathBuf::from("tests/data/files/noconfig_Scarb.toml")),
        )
        .unwrap_err();
        assert!(config.to_string().contains("Property not found in tool: rpc_url"));
    }

    #[test]
    fn test_parse_scarb_config_no_profile_found() {
        let config = parse_scarb_config(
            Some(String::from("mariusz")),
            Some(Utf8PathBuf::from("tests/data/contracts/balance/Scarb.toml")),
        )
        .unwrap_err();
        assert!(config.to_string().contains("Profile or property not found in Scarb.toml: mariusz, rpc_url"));
    }

    #[test]
    fn test_parse_scarb_config_account_missing() {
        let config = parse_scarb_config(
            None,
            Some(Utf8PathBuf::from("tests/data/files/somemissing_Scarb.toml")),
        )
        .unwrap_err();
        assert!(config.to_string().contains("Property not found in tool: account"));
    }
}
