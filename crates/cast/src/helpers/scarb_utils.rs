use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use scarb_metadata::{self, PackageMetadata};
use scarb_metadata::{Metadata, MetadataCommand};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::{Command, Stdio};
use std::str::FromStr;

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct CastConfig {
    pub rpc_url: String,
    pub account: String,
    pub accounts_file: Utf8PathBuf,
    pub keystore: Utf8PathBuf,
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
        })
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

pub fn get_property<'a, T>(tool: &'a Value, field: &str) -> T
where
    T: From<&'a str> + Default,
{
    tool.get(field)
        .and_then(Value::as_str)
        .map(T::from)
        .unwrap_or_default()
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

pub fn get_scarb_metadata(manifest_path: &Utf8PathBuf) -> Result<Metadata> {
    which::which("scarb")
        .context("Cannot find `scarb` binary in PATH. Make sure you have Scarb installed https://github.com/software-mansion/scarb")?;

    MetadataCommand::new()
        .inherit_stderr()
        .manifest_path(manifest_path)
        .exec()
        .context(format!(
            "Failed to read Scarb.toml manifest file, not found in {} nor parent directories.",
            manifest_path.clone().into_string()
        ))
}

pub fn parse_scarb_config(
    profile: &Option<String>,
    package_metadata: Option<&PackageMetadata>,
) -> Result<CastConfig> {
    match package_metadata {
        Some(data) => match get_package_tool_sncast(data) {
            Ok(package_tool_sncast) => {
                CastConfig::from_package_tool_sncast(package_tool_sncast, profile)
            }
            Err(_) => Ok(CastConfig::default()),
        },
        None => Ok(CastConfig::default()),
    }
}

pub fn get_package_tool_sncast(package: &PackageMetadata) -> Result<&Value> {
    let tool = package
        .manifest_metadata
        .tool
        .as_ref()
        .ok_or_else(|| anyhow!("No field [tool] found in package"))?;

    let tool_sncast = tool
        .get("sncast")
        .ok_or_else(|| anyhow!("No field [tool.sncast] found in package"))?;

    Ok(tool_sncast)
}

pub fn get_first_package_from_metadata(metadata: &Metadata) -> Result<&PackageMetadata> {
    let first_package_id = metadata
        .workspace
        .members
        .get(0)
        .ok_or_else(|| anyhow!("No package found in metadata"))?;

    let first_package = metadata
        .packages
        .iter()
        .find(|p| p.id == *first_package_id)
        .ok_or_else(|| anyhow!("No package found in metadata"))?;

    Ok(first_package)
}

#[cfg(test)]
mod tests {
    use crate::helpers::scarb_utils::get_first_package_from_metadata;
    use crate::helpers::scarb_utils::get_scarb_metadata;
    use crate::helpers::scarb_utils::parse_scarb_config;
    use camino::Utf8PathBuf;

    #[test]
    fn test_parse_scarb_config_happy_case_with_profile() {
        let metadata = get_scarb_metadata(&Utf8PathBuf::from(
            "tests/data/contracts/constructor_with_params/Scarb.toml",
        ))
        .unwrap();
        let config = parse_scarb_config(
            &Some(String::from("myprofile")),
            Some(get_first_package_from_metadata(&metadata).unwrap()),
        )
        .unwrap();

        assert_eq!(config.account, String::from("user1"));
        assert_eq!(config.rpc_url, String::from("http://127.0.0.1:5055/rpc"));
    }

    #[test]
    fn test_parse_scarb_config_happy_case_without_profile() {
        let metadata =
            get_scarb_metadata(&Utf8PathBuf::from("tests/data/contracts/map/Scarb.toml")).unwrap();
        let config = parse_scarb_config(
            &None,
            Some(get_first_package_from_metadata(&metadata).unwrap()),
        )
        .unwrap();
        assert_eq!(config.account, String::from("user2"));
        assert_eq!(config.rpc_url, String::from("http://127.0.0.1:5055/rpc"));
    }

    #[test]
    fn test_parse_scarb_config_not_in_file() {
        let metadata =
            get_scarb_metadata(&Utf8PathBuf::from("tests/data/files/noconfig_Scarb.toml")).unwrap();
        let config = parse_scarb_config(
            &None,
            Some(get_first_package_from_metadata(&metadata).unwrap()),
        )
        .unwrap();

        assert!(config.rpc_url.is_empty());
        assert!(config.account.is_empty());
    }

    #[test]
    fn test_parse_scarb_config_no_profile_found() {
        let metadata =
            get_scarb_metadata(&Utf8PathBuf::from("tests/data/contracts/map/Scarb.toml")).unwrap();
        let config = parse_scarb_config(
            &Some(String::from("mariusz")),
            Some(get_first_package_from_metadata(&metadata).unwrap()),
        )
        .unwrap_err();
        assert_eq!(
            config.to_string(),
            "No field [tool.sncast.mariusz] found in package"
        );
    }

    #[test]
    fn test_parse_scarb_config_account_missing() {
        let metadata = get_scarb_metadata(&Utf8PathBuf::from(
            "tests/data/files/somemissing_Scarb.toml",
        ))
        .unwrap();
    
        let config = parse_scarb_config(
            &None,
            Some(get_first_package_from_metadata(&metadata).unwrap()),
        )
        .unwrap();

        assert!(config.account.is_empty());
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
