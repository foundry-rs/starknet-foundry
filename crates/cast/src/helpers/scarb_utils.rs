use anyhow::{anyhow, bail, Context, Error, Result};
use camino::Utf8PathBuf;
use scarb_metadata;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::process::{Command, Stdio};
use std::str::FromStr;

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct CastConfig {
    pub rpc_url: String,
    pub network: String,
    pub account: String,
}

pub fn get_property(
    tool: &BTreeMap<String, Value>,
    profile: &Option<String>,
    property: &str,
) -> Result<String, Error> {
    let tool_sncast = tool
        .get("sncast")
        .ok_or_else(|| anyhow!("No tool.sncast"))?;

    match profile {
        Some(ref p) => tool_sncast
            .get(p)
            .and_then(|t| t.get(property))
            .and_then(serde_json::Value::as_str)
            .map(String::from)
            .ok_or(anyhow!(
                "Profile or property not found in Scarb.toml: {p}, {property}"
            )),
        None => match tool_sncast.get(property) {
            Some(property) => Ok(String::from(
                property.as_str().expect("Couldn't cast property to &str"),
            )),
            None => Ok(String::new()),
        },
    }
}

pub fn get_scarb_manifest() -> Result<Utf8PathBuf> {
    which::which("scarb")
        .context("Cannot find `scarb` binary in PATH. Make sure you have Scarb installed https://github.com/software-mansion/scarb")?;

    let output = Command::new("scarb")
        .arg("manifest-path")
        .stdout(Stdio::piped())
        .output()
        .context("Failed to execute scarb manifest-path command")?;

    let output_str = String::from_utf8(output.stdout)
        .context("Invalid output of scarb manifest-path command")?;

    let path = Utf8PathBuf::from_str(output_str.trim())
        .context("Scarb manifest-path returned invalid path")?;

    Ok(path)
}

pub fn parse_scarb_config(
    profile: &Option<String>,
    path: &Option<Utf8PathBuf>,
) -> Result<CastConfig> {
    if let Some(path) = path {
        if !path.exists() {
            bail!("{path} file does not exist!");
        }
    }

    let manifest_path = path.clone().unwrap_or_else(|| {
        get_scarb_manifest().expect("Failed to obtain manifest path from scarb")
    });

    if !manifest_path.exists() {
        return Ok(CastConfig::default());
    }

    let metadata = scarb_metadata::MetadataCommand::new()
        .inherit_stderr()
        .manifest_path(manifest_path)
        .no_deps()
        .exec()
        .context(
            "Failed to read Scarb.toml manifest file, not found in current nor parent directories",
        )?;

    let package_tool_result = get_package_tool(&metadata);
    if package_tool_result.is_err() {
        return Ok(CastConfig::default());
    }

    let package_tool = package_tool_result?;

    let rpc_url = get_property(package_tool, profile, "url")?;
    let network = get_property(package_tool, profile, "network")?;
    let account = get_property(package_tool, profile, "account")?;

    Ok(CastConfig {
        rpc_url,
        network,
        account,
    })
}

pub fn get_package_tool(metadata: &scarb_metadata::Metadata) -> Result<&BTreeMap<String, Value>> {
    let first_package = metadata
        .packages
        .get(0)
        .ok_or_else(|| anyhow!("No package found in metadata"))?;

    let tool = first_package
        .manifest_metadata
        .tool
        .as_ref()
        .ok_or_else(|| anyhow!("No tool found in package"))?;

    Ok(tool)
}

#[cfg(test)]
mod tests {
    use crate::helpers::scarb_utils::parse_scarb_config;
    use camino::Utf8PathBuf;
    use sealed_test::prelude::rusty_fork_test;
    use sealed_test::prelude::sealed_test;

    #[test]
    fn test_parse_scarb_config_happy_case_with_profile() {
        let config = parse_scarb_config(
            &Some(String::from("myprofile")),
            &Some(Utf8PathBuf::from(
                "tests/data/contracts/v1/balance/Scarb.toml",
            )),
        )
        .unwrap();

        assert_eq!(config.account, String::from("user1"));
        assert_eq!(config.network, String::from("testnet"));
        assert_eq!(config.rpc_url, String::from("http://127.0.0.1:5055/rpc"));
    }

    #[test]
    fn test_parse_scarb_config_happy_case_without_profile() {
        let config = parse_scarb_config(
            &None,
            &Some(Utf8PathBuf::from("tests/data/contracts/v1/map/Scarb.toml")),
        )
        .unwrap();
        assert_eq!(config.account, String::from("user2"));
        assert_eq!(config.network, String::from("testnet"));
        assert_eq!(config.rpc_url, String::from("http://127.0.0.1:5055/rpc"));
    }

    #[test]
    fn test_parse_scarb_config_not_found() {
        let config =
            parse_scarb_config(&None, &Some(Utf8PathBuf::from("whatever/Scarb.toml"))).unwrap_err();
        assert!(config.to_string().contains("file does not exist!"));
    }

    #[test]
    fn test_parse_scarb_config_no_path_not_found() {
        let config = parse_scarb_config(&None, &None).unwrap();

        assert!(config.rpc_url.is_empty());
        assert!(config.network.is_empty());
        assert!(config.account.is_empty());
    }

    #[test]
    fn test_parse_scarb_config_not_in_file() {
        let config = parse_scarb_config(
            &None,
            &Some(Utf8PathBuf::from("tests/data/files/noconfig_Scarb.toml")),
        )
        .unwrap();

        assert!(config.rpc_url.is_empty());
        assert!(config.network.is_empty());
        assert!(config.account.is_empty());
    }

    #[test]
    fn test_parse_scarb_config_no_profile_found() {
        let config = parse_scarb_config(
            &Some(String::from("mariusz")),
            &Some(Utf8PathBuf::from(
                "tests/data/contracts/v1/balance/Scarb.toml",
            )),
        )
        .unwrap_err();
        assert!(config
            .to_string()
            .contains("Profile or property not found in Scarb.toml: mariusz, url"));
    }

    #[test]
    fn test_parse_scarb_config_account_missing() {
        let config = parse_scarb_config(
            &None,
            &Some(Utf8PathBuf::from("tests/data/files/somemissing_Scarb.toml")),
        )
        .unwrap();

        assert!(config.account.is_empty());
    }

    #[sealed_test(files = ["tests/data/contracts/v1/balance/Scarb.toml"])]
    fn test_parse_scarb_config_no_profile_no_path() {
        let config = parse_scarb_config(&None, &None).unwrap();

        assert!(config.rpc_url.is_empty());
        assert!(config.network.is_empty());
        assert!(config.account.is_empty());
    }

    #[sealed_test(files = ["tests/data/contracts/v1/balance/Scarb.toml"])]
    fn test_parse_scarb_config_no_path() {
        let config = parse_scarb_config(&Some(String::from("myprofile")), &None).unwrap();

        assert_eq!(config.rpc_url, String::from("http://127.0.0.1:5055/rpc"));
        assert_eq!(config.network, String::from("testnet"));
        assert_eq!(config.account, String::from("user1"));
    }
}
