use super::constants::{CONFIG_FILENAME, WAIT_RETRY_INTERVAL, WAIT_TIMEOUT};
use crate::ValidatedWaitParams;
use anyhow::{anyhow, Result};
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use std::fs;
use tempfile::TempDir;
use toml::Value;

#[derive(Default, Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct CastConfig {
    pub rpc_url: String,
    pub account: String,
    pub accounts_file: Utf8PathBuf,
    pub keystore: Option<Utf8PathBuf>,
    pub wait_params: ValidatedWaitParams,
}

impl CastConfig {
    pub fn from_sncast_config(
        sncast_config: &Value,
        profile: &Option<String>,
    ) -> Result<CastConfig> {
        let entries = get_profile(sncast_config, profile)?;

        Ok(CastConfig {
            rpc_url: get_property(&entries, "url"),
            account: get_property(&entries, "account"),
            accounts_file: get_property(&entries, "accounts-file"),
            keystore: get_property_optional(&entries, "keystore"),
            wait_params: ValidatedWaitParams::new(
                get_property(&entries, "wait-retry-interval"),
                get_property(&entries, "wait-timeout"),
            ),
        })
    }
}

pub trait PropertyFromCastConfig: Sized {
    fn from_toml_value(value: &Value) -> Option<Self>;
    fn default_value() -> Self;
}

impl PropertyFromCastConfig for String {
    fn from_toml_value(value: &Value) -> Option<Self> {
        value.as_str().map(std::borrow::ToOwned::to_owned)
    }

    fn default_value() -> Self {
        String::default()
    }
}

impl PropertyFromCastConfig for Utf8PathBuf {
    fn from_toml_value(value: &Value) -> Option<Self> {
        value.as_str().map(Utf8PathBuf::from)
    }

    fn default_value() -> Self {
        Utf8PathBuf::default()
    }
}

impl PropertyFromCastConfig for u8 {
    fn from_toml_value(value: &Value) -> Option<Self> {
        value.as_integer().and_then(|i| i.try_into().ok())
    }

    fn default_value() -> Self {
        WAIT_RETRY_INTERVAL
    }
}

impl PropertyFromCastConfig for u16 {
    fn from_toml_value(value: &Value) -> Option<Self> {
        value.as_integer().and_then(|i| i.try_into().ok())
    }

    fn default_value() -> Self {
        WAIT_TIMEOUT
    }
}

impl<T> PropertyFromCastConfig for Option<T>
where
    T: PropertyFromCastConfig,
{
    fn from_toml_value(value: &Value) -> Option<Self> {
        T::from_toml_value(value).map(Some)
    }
    fn default_value() -> Self {
        Some(T::default_value())
    }
}

pub fn get_profile(sncast_config: &Value, profile: &Option<String>) -> Result<Value> {
    let profile_name = profile.as_deref().unwrap_or("default");
    let config = sncast_config
        .get("sncast")
        .expect("Failed to find sncast config in snfoundry.toml file");

    match config.get(profile_name) {
        Some(profile_value) => Ok(profile_value.clone()),
        None if profile_name == "default" => Ok(Value::Table(Default::default())),
        None => Err(anyhow!("Profile [{}] not found in config", profile_name)),
    }
}

pub fn get_property<T>(entries: &Value, field: &str) -> T
where
    T: PropertyFromCastConfig + Default,
{
    get_property_optional(entries, field).unwrap_or_else(T::default_value)
}

pub fn get_property_optional<T>(entries: &Value, field: &str) -> Option<T>
where
    T: PropertyFromCastConfig + Default,
{
    entries.get(field).and_then(T::from_toml_value)
}

pub fn load_config(profile: &Option<String>, path: &Option<Utf8PathBuf>) -> Result<CastConfig> {
    let config_path = path
        .as_ref()
        .and_then(|p| search_config_upwards_relative_to(p).ok())
        .or_else(|| find_config_file().ok());

    match config_path {
        Some(path) => {
            let config = fs::read_to_string(path)
                .expect("Failed to read snfoundry.toml config file")
                .parse::<Value>()
                .expect("Failed to parse snfoundry.toml config file");

            CastConfig::from_sncast_config(&config, profile)
        }
        None => Ok(CastConfig::default()),
    }
}

pub fn search_config_upwards_relative_to(current_dir: &Utf8PathBuf) -> Result<Utf8PathBuf> {
    current_dir
        .ancestors()
        .find(|path| fs::metadata(path.join(CONFIG_FILENAME)).is_ok())
        .map(|path| path.join(CONFIG_FILENAME))
        .ok_or_else(|| {
            anyhow!(
                "Failed to find snfoundry.toml - not found in current nor any parent directories"
            )
        })
}

pub fn find_config_file() -> Result<Utf8PathBuf> {
    search_config_upwards_relative_to(&Utf8PathBuf::try_from(
        std::env::current_dir().expect("Failed to get current directory"),
    )?)
}

#[must_use]
pub fn copy_config_to_tempdir(src_path: &str, additional_path: Option<&str>) -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create a temporary directory");
    if let Some(dir) = additional_path {
        let path = temp_dir.path().join(dir);
        fs::create_dir_all(path).expect("Failed to create directories in temp dir");
    };
    let temp_dir_file_path = temp_dir.path().join(CONFIG_FILENAME);
    fs::copy(src_path, temp_dir_file_path).expect("Failed to copy config file to temp dir");
    temp_dir
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn find_config_in_current_dir() {
        let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
        let path = search_config_upwards_relative_to(
            &Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap(),
        )
        .unwrap();
        assert_eq!(path, tempdir.path().join(CONFIG_FILENAME));
    }

    #[test]
    fn find_config_in_parent_dir() {
        let tempdir =
            copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", Some("childdir"));
        let path = search_config_upwards_relative_to(
            &Utf8PathBuf::try_from(tempdir.path().to_path_buf().join("childdir")).unwrap(),
        )
        .unwrap();
        assert_eq!(path, tempdir.path().join(CONFIG_FILENAME));
    }

    #[test]
    fn find_config_in_parent_dir_two_levels() {
        let tempdir = copy_config_to_tempdir(
            "tests/data/files/correct_snfoundry.toml",
            Some("childdir1/childdir2"),
        );
        let path = search_config_upwards_relative_to(
            &Utf8PathBuf::try_from(tempdir.path().to_path_buf().join("childdir1/childdir2"))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(path, tempdir.path().join(CONFIG_FILENAME));
    }

    #[test]
    fn find_config_in_parent_dir_available_in_multiple_parents() {
        let tempdir =
            copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", Some("childdir1"));
        fs::copy(
            "tests/data/files/correct_snfoundry.toml",
            tempdir.path().join("childdir1").join(CONFIG_FILENAME),
        )
        .expect("Failed to copy config file to temp dir");
        let path = search_config_upwards_relative_to(
            &Utf8PathBuf::try_from(tempdir.path().to_path_buf().join("childdir1")).unwrap(),
        )
        .unwrap();
        assert_eq!(path, tempdir.path().join("childdir1").join(CONFIG_FILENAME));
    }

    #[test]
    fn no_config_in_current_nor_parent_dir() {
        let tempdir = TempDir::new().expect("Failed to create a temporary directory");
        assert!(
            search_config_upwards_relative_to(
                &Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap()
            )
            .is_err(),
            "Failed to find snfoundry.toml - not found in current nor any parent directories"
        );
    }

    #[test]
    fn load_config_happy_case_with_profile() {
        let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
        let config = load_config(
            &Some(String::from("profile1")),
            &Some(Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap()),
        )
        .unwrap();

        assert_eq!(config.account, String::from("user3"));
        assert_eq!(config.rpc_url, String::from("http://127.0.0.1:5050/rpc"));
    }

    #[test]
    fn load_config_happy_case_default_profile() {
        let tempdir = copy_config_to_tempdir("tests/data/files/correct_snfoundry.toml", None);
        let config = load_config(
            &None,
            &Some(Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap()),
        )
        .unwrap();
        assert_eq!(config.account, String::from("user1"));
        assert_eq!(config.rpc_url, String::from("http://127.0.0.1:5055/rpc"));
    }

    #[test]
    fn load_config_not_found() {
        let tempdir = TempDir::new().expect("Failed to create a temporary directory");
        let config = load_config(
            &None,
            &Some(Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap()),
        )
        .unwrap();

        assert_eq!(config.account, String::new());
        assert_eq!(config.rpc_url, String::new());
    }

    #[test]
    fn test_config_defaults() {
        let config = CastConfig::default();
        assert_eq!(config.wait_params.get_timeout(), WAIT_TIMEOUT);
        assert_eq!(config.wait_params.get_retry_interval(), WAIT_RETRY_INTERVAL);
    }
}
