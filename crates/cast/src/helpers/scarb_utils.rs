use super::constants::{WAIT_RETRY_INTERVAL, WAIT_TIMEOUT};
use anyhow::{anyhow, bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use scarb_metadata;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::default::Default;
use std::env;
use std::fs::canonicalize;
use std::process::{Command, Stdio};
use std::str::FromStr;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CastConfig {
    pub rpc_url: String,
    pub account: String,
    pub accounts_file: Utf8PathBuf,
    pub keystore: Utf8PathBuf,
    pub wait_timeout: u16,
    pub wait_retry_interval: u8,
}

impl CastConfig {
    pub fn from_package_tool_sncast(
        package_tool_sncast: &Value,
        profile: &Option<String>,
    ) -> Result<CastConfig> {
        let tool = get_profile(package_tool_sncast, profile)?;

        Ok(CastConfig {
            rpc_url: get_property(tool, "url"),
            account: get_property(tool, "account"),
            accounts_file: get_property(tool, "accounts-file"),
            keystore: get_property(tool, "keystore"),
            wait_timeout: get_property(tool, "wait_timeout"),
            wait_retry_interval: get_property(tool, "wait_retry_interval"),
        })
    }
}

impl Default for CastConfig {
    fn default() -> Self {
        Self {
            rpc_url: String::default(),
            account: String::default(),
            accounts_file: Utf8PathBuf::default(),
            keystore: Utf8PathBuf::default(),
            wait_timeout: WAIT_TIMEOUT,
            wait_retry_interval: WAIT_RETRY_INTERVAL,
        }
    }
}

pub trait PropertyFromCastConfig: Sized {
    fn from_toml_value(value: &Value) -> Option<Self>;
}

impl PropertyFromCastConfig for String {
    fn from_toml_value(value: &Value) -> Option<Self> {
        value.as_str().map(std::borrow::ToOwned::to_owned)
    }
}

impl PropertyFromCastConfig for Utf8PathBuf {
    fn from_toml_value(value: &Value) -> Option<Self> {
        value.as_str().map(Utf8PathBuf::from)
    }
}

impl PropertyFromCastConfig for u8 {
    fn from_toml_value(value: &Value) -> Option<Self> {
        value.as_u64().and_then(|i| i.try_into().ok())
    }
}

impl PropertyFromCastConfig for u16 {
    fn from_toml_value(value: &Value) -> Option<Self> {
        value.as_u64().and_then(|i| i.try_into().ok())
    }
}

impl<T> PropertyFromCastConfig for Option<T>
where
    T: PropertyFromCastConfig,
{
    fn from_toml_value(value: &Value) -> Option<Self> {
        T::from_toml_value(value).map(Some)
    }
}

pub fn get_profile<'a>(tool_sncast: &'a Value, profile: &Option<String>) -> Result<&'a Value> {
    match profile {
        Some(profile_) => tool_sncast
            .get(profile_)
            .ok_or_else(|| anyhow!("No field [tool.sncast.{}] found in package", profile_)),
        None => Ok(tool_sncast),
    }
}

pub fn get_property<T>(tool: &Value, field: &str) -> T
where
    T: PropertyFromCastConfig + Default,
{
    tool.get(field)
        .and_then(T::from_toml_value)
        .unwrap_or_default()
}

pub fn get_scarb_manifest() -> Result<Utf8PathBuf> {
    get_scarb_manifest_for(<&Utf8Path>::from("."))
}

pub fn get_scarb_manifest_for(dir: &Utf8Path) -> Result<Utf8PathBuf> {
    which::which("scarb")
        .context("Cannot find `scarb` binary in PATH. Make sure you have Scarb installed https://github.com/software-mansion/scarb")?;

    let output = Command::new("scarb")
        .current_dir(dir)
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

fn get_scarb_metadata_command(
    manifest_path: &Utf8PathBuf,
) -> Result<scarb_metadata::MetadataCommand> {
    which::which("scarb")
        .context("Cannot find `scarb` binary in PATH. Make sure you have Scarb installed https://github.com/software-mansion/scarb")?;

    let mut command = scarb_metadata::MetadataCommand::new();
    command.inherit_stderr().manifest_path(manifest_path);
    Ok(command)
}

fn execute_scarb_metadata_command(
    command: &scarb_metadata::MetadataCommand,
) -> Result<scarb_metadata::Metadata> {
    command.exec().context(format!(
        "Failed to read Scarb.toml manifest file, not found in current nor parent directories, {}",
        env::current_dir()
            .unwrap()
            .into_os_string()
            .into_string()
            .unwrap()
    ))
}

pub fn get_scarb_metadata(manifest_path: &Utf8PathBuf) -> Result<scarb_metadata::Metadata> {
    let mut command = get_scarb_metadata_command(manifest_path)?;
    let command = command.no_deps();
    execute_scarb_metadata_command(command)
}

pub fn get_scarb_metadata_with_deps(
    manifest_path: &Utf8PathBuf,
) -> Result<scarb_metadata::Metadata> {
    let command = get_scarb_metadata_command(manifest_path)?;
    execute_scarb_metadata_command(&command)
}

#[must_use]
pub fn verify_or_determine_scarb_manifest_path(
    path_to_scarb_toml: &Option<Utf8PathBuf>,
) -> Option<Utf8PathBuf> {
    if let Some(path) = path_to_scarb_toml {
        assert!(path.exists(), "{path} file does not exist!");
    }

    let manifest_path = match path_to_scarb_toml.clone() {
        Some(path) => path,
        None => get_scarb_manifest()
            .context("Failed to obtain manifest path from scarb")
            .unwrap(),
    };

    if !manifest_path.exists() {
        return None;
    }

    Some(manifest_path)
}

pub fn get_package_metadata<'a>(
    metadata: &'a scarb_metadata::Metadata,
    manifest_path: &'a Utf8PathBuf,
) -> Result<&'a scarb_metadata::PackageMetadata> {
    let manifest_path = canonicalize(manifest_path.clone())
        .unwrap_or_else(|err| panic!("Failed to canonicalize {manifest_path}, error: {err:?}"));

    let package = metadata
        .packages
        .iter()
        .find(|package| package.manifest_path == manifest_path)
        .ok_or(anyhow!(
            "Path {} not found in scarb metadata",
            manifest_path.display()
        ))?;
    Ok(package)
}

pub fn parse_scarb_config(
    profile: &Option<String>,
    path: &Option<Utf8PathBuf>,
) -> Result<CastConfig> {
    let manifest_path = match path.clone() {
        Some(path) => {
            if !(path.exists()) {
                bail!("{path} file does not exist!");
            }
            path
        }
        None => get_scarb_manifest().context("Failed to obtain manifest path from scarb")?,
    };

    if !manifest_path.exists() {
        return Ok(CastConfig::default());
    }

    let metadata = get_scarb_metadata(&manifest_path)?;

    match get_package_tool_sncast(&metadata) {
        Ok(package_tool_sncast) => {
            CastConfig::from_package_tool_sncast(package_tool_sncast, profile)
        }
        Err(_) => Ok(CastConfig::default()),
    }
}

pub fn get_package_tool_sncast(metadata: &scarb_metadata::Metadata) -> Result<&Value> {
    let first_package = metadata
        .packages
        .get(0)
        .ok_or_else(|| anyhow!("No package found in metadata"))?;

    let tool = first_package
        .manifest_metadata
        .tool
        .as_ref()
        .ok_or_else(|| anyhow!("No field [tool] found in package"))?;

    let tool_sncast = tool
        .get("sncast")
        .ok_or_else(|| anyhow!("No field [tool.sncast] found in package"))?;

    Ok(tool_sncast)
}

#[cfg(test)]
mod tests {
    use crate::helpers::scarb_utils::get_scarb_metadata;
    use crate::helpers::scarb_utils::parse_scarb_config;
    use camino::Utf8PathBuf;
    use sealed_test::prelude::rusty_fork_test;
    use sealed_test::prelude::sealed_test;

    #[test]
    fn test_parse_scarb_config_happy_case_with_profile() {
        let config = parse_scarb_config(
            &Some(String::from("myprofile")),
            &Some(Utf8PathBuf::from(
                "tests/data/contracts/constructor_with_params/Scarb.toml",
            )),
        )
        .unwrap();

        assert_eq!(config.account, String::from("user1"));
        assert_eq!(config.rpc_url, String::from("http://127.0.0.1:5055/rpc"));
    }

    #[test]
    fn test_parse_scarb_config_happy_case_without_profile() {
        let config = parse_scarb_config(
            &None,
            &Some(Utf8PathBuf::from("tests/data/contracts/map/Scarb.toml")),
        )
        .unwrap();
        assert_eq!(config.account, String::from("user2"));
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
        assert!(config.account.is_empty());
    }

    #[test]
    fn test_parse_scarb_config_no_profile_found() {
        let config = parse_scarb_config(
            &Some(String::from("mariusz")),
            &Some(Utf8PathBuf::from("tests/data/contracts/map/Scarb.toml")),
        )
        .unwrap_err();
        assert_eq!(
            config.to_string(),
            "No field [tool.sncast.mariusz] found in package"
        );
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

    #[sealed_test(files = ["tests/data/contracts/no_sierra/Scarb.toml"])]
    fn test_parse_scarb_config_no_profile_no_path() {
        let config = parse_scarb_config(&None, &None).unwrap();

        assert!(config.rpc_url.is_empty());
        assert!(config.account.is_empty());
    }

    #[sealed_test(files = ["tests/data/contracts/constructor_with_params/Scarb.toml"])]
    fn test_parse_scarb_config_no_path() {
        let config = parse_scarb_config(&Some(String::from("myprofile")), &None).unwrap();

        assert_eq!(config.rpc_url, String::from("http://127.0.0.1:5055/rpc"));
        assert_eq!(config.account, String::from("user1"));
    }

    #[test]
    fn test_get_scarb_metadata() {
        let metadata = get_scarb_metadata(&"tests/data/contracts/map/Scarb.toml".into());
        assert!(metadata.is_ok());
    }

    #[test]
    fn test_get_scarb_metadata_not_found() {
        let metadata_err = get_scarb_metadata(&"Scarb.toml".into()).unwrap_err();
        assert!(metadata_err
            .to_string()
            .contains("Failed to read Scarb.toml manifest file"));
    }
}
