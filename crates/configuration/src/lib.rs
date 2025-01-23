use anyhow::{anyhow, Context, Result};
use scarb_metadata::{Metadata, PackageId};
use serde_json::Number;
use std::{env, fs};

use camino::Utf8PathBuf;
use tempfile::{tempdir, TempDir};
use toml::Value;
pub const CONFIG_FILENAME: &str = "snfoundry.toml";

/// Defined in snfoundry.toml
/// Configuration not associated with any specific package
pub trait Config {
    #[must_use]
    fn tool_name() -> &'static str;

    fn from_raw(config: serde_json::Value) -> Result<Self>
    where
        Self: Sized;
}

/// Defined in scarb manifest
/// Configuration associated with a specific package
pub trait PackageConfig {
    #[must_use]
    fn tool_name() -> &'static str;

    fn from_raw(config: &serde_json::Value) -> Result<Self>
    where
        Self: Sized;
}

fn get_with_ownership(config: serde_json::Value, key: &str) -> Option<serde_json::Value> {
    match config {
        serde_json::Value::Object(mut map) => map.remove(key),
        _ => None,
    }
}

pub fn get_profile(
    raw_config: serde_json::Value,
    tool: &str,
    profile: Option<&str>,
) -> Result<serde_json::Value> {
    let profile_name = profile.unwrap_or("default");
    let tool_config = get_with_ownership(raw_config, tool)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

    match get_with_ownership(tool_config, profile_name) {
        Some(profile_value) => Ok(profile_value),
        None if profile_name == "default" => Ok(serde_json::Value::Object(Default::default())),
        None => Err(anyhow!("Profile [{}] not found in config", profile_name)),
    }
}

pub fn load_config<T: Config + Default>(
    path: Option<&Utf8PathBuf>,
    profile: Option<&str>,
) -> Result<T> {
    let config_path = path
        .as_ref()
        .and_then(|p| search_config_upwards_relative_to(p).ok())
        .or_else(|| find_config_file().ok());

    match config_path {
        Some(path) => {
            let raw_config_toml = fs::read_to_string(path)
                .context("Failed to read snfoundry.toml config file")?
                .parse::<Value>()
                .context("Failed to parse snfoundry.toml config file")?;

            let raw_config_json = serde_json::to_value(raw_config_toml)
                .context("Conversion from TOML value to JSON value should not fail.")?;

            let profile = get_profile(raw_config_json, T::tool_name(), profile)?;
            T::from_raw(resolve_env_variables(profile)?)
        }
        None => Ok(T::default()),
    }
}
/// Loads config for a specific package from the `Scarb.toml` file
/// # Arguments
/// * `metadata` - Scarb metadata object
/// * `package` - Id of the Scarb package
pub fn load_package_config<T: PackageConfig + Default>(
    metadata: &Metadata,
    package: &PackageId,
) -> Result<T> {
    let maybe_raw_metadata = metadata
        .get_package(package)
        .ok_or_else(|| anyhow!("Failed to find metadata for package = {package}"))?
        .tool_metadata(T::tool_name())
        .cloned();
    match maybe_raw_metadata {
        Some(raw_metadata) => T::from_raw(&resolve_env_variables(raw_metadata)?),
        None => Ok(T::default()),
    }
}

fn resolve_env_variables(config: serde_json::Value) -> Result<serde_json::Value> {
    match config {
        serde_json::Value::Object(map) => {
            let val = map
                .into_iter()
                .map(|(k, v)| -> Result<(String, serde_json::Value)> {
                    Ok((k, resolve_env_variables(v)?))
                })
                .collect::<Result<serde_json::Map<String, serde_json::Value>>>()?;
            Ok(serde_json::Value::Object(val))
        }
        serde_json::Value::Array(val) => {
            let val = val
                .into_iter()
                .map(resolve_env_variables)
                .collect::<Result<Vec<serde_json::Value>>>()?;
            Ok(serde_json::Value::Array(val))
        }
        serde_json::Value::String(val) if val.starts_with('$') => resolve_env_variable(&val),
        val => Ok(val),
    }
}

fn resolve_env_variable(var: &str) -> Result<serde_json::Value> {
    assert!(var.starts_with('$'));
    let mut initial_value = &var[1..];
    if initial_value.starts_with('{') && initial_value.ends_with('}') {
        initial_value = &initial_value[1..initial_value.len() - 1];
    }
    let value = env::var(&initial_value)?;

    if let Ok(value) = value.parse::<Number>() {
        return Ok(serde_json::Value::Number(value));
    }
    if let Ok(value) = value.parse::<bool>() {
        return Ok(serde_json::Value::Bool(value));
    }
    Ok(serde_json::Value::String(value))
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
        env::current_dir().expect("Failed to get current directory"),
    )?)
}

pub fn copy_config_to_tempdir(src_path: &str, additional_path: Option<&str>) -> Result<TempDir> {
    let temp_dir = tempdir().context("Failed to create a temporary directory")?;
    if let Some(dir) = additional_path {
        let path = temp_dir.path().join(dir);
        fs::create_dir_all(path).context("Failed to create directories in temp dir")?;
    };
    let temp_dir_file_path = temp_dir.path().join(CONFIG_FILENAME);
    fs::copy(src_path, temp_dir_file_path).context("Failed to copy config file to temp dir")?;

    Ok(temp_dir)
}

#[cfg(test)]
mod tests {
    use std::fs::{self, File};

    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn find_config_in_current_dir() {
        let tempdir = copy_config_to_tempdir("tests/data/stubtool_snfoundry.toml", None).unwrap();
        let path = search_config_upwards_relative_to(
            &Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap(),
        )
        .unwrap();
        assert_eq!(path, tempdir.path().join(CONFIG_FILENAME));
    }

    #[test]
    fn find_config_in_parent_dir() {
        let tempdir =
            copy_config_to_tempdir("tests/data/stubtool_snfoundry.toml", Some("childdir")).unwrap();
        let path = search_config_upwards_relative_to(
            &Utf8PathBuf::try_from(tempdir.path().to_path_buf().join("childdir")).unwrap(),
        )
        .unwrap();
        assert_eq!(path, tempdir.path().join(CONFIG_FILENAME));
    }

    #[test]
    fn find_config_in_parent_dir_two_levels() {
        let tempdir = copy_config_to_tempdir(
            "tests/data/stubtool_snfoundry.toml",
            Some("childdir1/childdir2"),
        )
        .unwrap();
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
            copy_config_to_tempdir("tests/data/stubtool_snfoundry.toml", Some("childdir1"))
                .unwrap();
        fs::copy(
            "tests/data/stubtool_snfoundry.toml",
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
        let tempdir = tempdir().expect("Failed to create a temporary directory");
        assert!(
            search_config_upwards_relative_to(
                &Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap()
            )
            .is_err(),
            "Failed to find snfoundry.toml - not found in current nor any parent directories"
        );
    }

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct StubConfig {
        #[serde(default)]
        pub url: String,
        #[serde(default)]
        pub account: String,
    }
    impl Config for StubConfig {
        fn tool_name() -> &'static str {
            "stubtool"
        }

        fn from_raw(config: serde_json::Value) -> Result<Self> {
            Ok(serde_json::from_value::<StubConfig>(config)?)
        }
    }
    #[test]
    fn load_config_happy_case_with_profile() {
        let tempdir = copy_config_to_tempdir("tests/data/stubtool_snfoundry.toml", None).unwrap();
        let config = load_config::<StubConfig>(
            Some(&Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap()),
            Some(&String::from("profile1")),
        )
        .unwrap();
        assert_eq!(config.account, String::from("user3"));
        assert_eq!(config.url, String::from("http://127.0.0.1:5050/rpc"));
    }

    #[test]
    fn load_config_happy_case_default_profile() {
        let tempdir = copy_config_to_tempdir("tests/data/stubtool_snfoundry.toml", None).unwrap();
        let config = load_config::<StubConfig>(
            Some(&Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap()),
            None,
        )
        .unwrap();
        assert_eq!(config.account, String::from("user1"));
        assert_eq!(config.url, String::from("http://127.0.0.1:5055/rpc"));
    }

    #[test]
    fn load_config_not_found() {
        let tempdir = tempdir().expect("Failed to create a temporary directory");
        let config = load_config::<StubConfig>(
            Some(&Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap()),
            None,
        )
        .unwrap();

        assert_eq!(config.account, String::new());
        assert_eq!(config.url, String::new());
    }

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct StubComplexConfig {
        #[serde(default)]
        pub url: String,
        #[serde(default)]
        pub account: i32,
        #[serde(default)]
        pub nested: StubComplexConfigNested,
    }

    #[derive(Debug, Default, Serialize, Deserialize)]
    pub struct StubComplexConfigNested {
        #[serde(
            default,
            rename(serialize = "list-example", deserialize = "list-example")
        )]
        list_example: Vec<bool>,
        #[serde(default, rename(serialize = "url-nested", deserialize = "url-nested"))]
        url_nested: f32,
        #[serde(default, rename(serialize = "url-alt", deserialize = "url-alt"))]
        url_alt: String,
    }

    impl Config for StubComplexConfig {
        fn tool_name() -> &'static str {
            "stubtool"
        }

        fn from_raw(config: serde_json::Value) -> Result<Self> {
            Ok(serde_json::from_value::<StubComplexConfig>(config)?)
        }
    }

    #[test]
    fn empty_config_works() {
        let temp_dir = tempdir().expect("Failed to create a temporary directory");
        File::create(temp_dir.path().join(CONFIG_FILENAME)).unwrap();

        load_config::<StubConfig>(
            Some(&Utf8PathBuf::try_from(temp_dir.path().to_path_buf()).unwrap()),
            None,
        )
        .unwrap();
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn resolve_env_vars() {
        let tempdir =
            copy_config_to_tempdir("tests/data/stubtool_snfoundry.toml", Some("childdir1"))
                .unwrap();
        fs::copy(
            "tests/data/stubtool_snfoundry.toml",
            tempdir.path().join("childdir1").join(CONFIG_FILENAME),
        )
        .expect("Failed to copy config file to temp dir");
        // missing env variables
        if load_config::<StubConfig>(
            Some(&Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap()),
            Some(&String::from("with-envs")),
        )
        .is_ok()
        {
            panic!("Expected failure");
        }

        // present env variables
        env::set_var("VALUE_STRING123132", "nfsaufbnsailfbsbksdabfnkl");
        env::set_var("VALUE_STRING123142", "nfsasnsidnnsailfbsbksdabdkdkl");
        env::set_var("VALUE_INT123132", "321312");
        env::set_var("VALUE_FLOAT123132", "321.312");
        env::set_var("VALUE_BOOL1231321", "true");
        env::set_var("VALUE_BOOL1231322", "false");
        let config = load_config::<StubComplexConfig>(
            Some(&Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap()),
            Some(&String::from("with-envs")),
        )
        .unwrap();
        assert_eq!(config.url, String::from("nfsaufbnsailfbsbksdabfnkl"));
        assert_eq!(config.account, 321_312);
        assert_eq!(config.nested.list_example, vec![true, false]);
        assert_eq!(config.nested.url_nested, 321.312);
        assert_eq!(config.nested.url_alt, String::from("nfsasnsidnnsailfbsbksdabdkdkl"));
    }
}
