use crate::core::Profile;
use anyhow::{Context, Result, anyhow};
use camino::Utf8PathBuf;
use std::fs::File;
use std::{env, fs};
use toml::Table;

pub mod core;
pub mod test_utils;

pub const CONFIG_FILENAME: &str = "snfoundry.toml";

/// Configuration not associated with any specific package
pub trait Config {
    #[must_use]
    fn tool_name() -> &'static str;

    fn from_raw(config: serde_json::Value) -> Result<Self>
    where
        Self: Sized;
}

pub trait Override {
    #[must_use]
    fn override_with(&self, other: Self) -> Self;
}

pub fn override_optional<T: Override>(base: Option<T>, other: Option<T>) -> Option<T> {
    match (base, other) {
        (Some(base), Some(other)) => Some(base.override_with(other)),
        (None, Some(other)) => Some(other),
        (base, None) => base,
    }
}

#[must_use]
pub fn resolve_config_file() -> Utf8PathBuf {
    find_config_file().unwrap_or_else(|_| {
        let path = Utf8PathBuf::from(CONFIG_FILENAME);
        File::create(&path).expect("creating file in current directory should be possible");

        path.canonicalize_utf8()
            .expect("path canonicalize in current directory should be possible")
    })
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
                .parse::<Table>()
                .context("Failed to parse snfoundry.toml config file")?;

            let raw_config_json = serde_json::to_value(raw_config_toml)
                .context("Conversion from TOML value to JSON value should not fail.")?;

            core::load_config(
                raw_config_json,
                profile.map_or_else(|| Profile::Default, |p| Profile::Some(p.to_string())),
            )
        }
        None => Ok(T::default()),
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
        env::current_dir().expect("Failed to get current directory"),
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::copy_config_to_tempdir;
    use serde::{Deserialize, Serialize};
    use std::fs::{self, File};
    use tempfile::tempdir;
    use url::Url;

    #[test]
    fn find_config_in_current_dir() {
        let tempdir = copy_config_to_tempdir("tests/data/stubtool_snfoundry.toml", None);
        let path = search_config_upwards_relative_to(
            &Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap(),
        )
        .unwrap();
        assert_eq!(path, tempdir.path().join(CONFIG_FILENAME));
    }

    #[test]
    fn find_config_in_parent_dir() {
        let tempdir =
            copy_config_to_tempdir("tests/data/stubtool_snfoundry.toml", Some("childdir"));
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
            copy_config_to_tempdir("tests/data/stubtool_snfoundry.toml", Some("childdir1"));
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
        pub url: Option<Url>,
        #[serde(default)]
        pub account: String,
    }
    impl Config for StubConfig {
        fn tool_name() -> &'static str {
            "stubtool"
        }

        fn from_raw(config: serde_json::Value) -> Result<Self> {
            Ok(serde_json::from_value::<Self>(config)?)
        }
    }
    #[test]
    fn load_config_happy_case_with_profile() {
        let tempdir = copy_config_to_tempdir("tests/data/stubtool_snfoundry.toml", None);
        let config = load_config::<StubConfig>(
            Some(&Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap()),
            Some(&String::from("profile1")),
        )
        .unwrap();
        assert_eq!(config.account, String::from("user3"));
        assert_eq!(
            config.url,
            Some(Url::parse("http://127.0.0.1:5050/rpc").unwrap())
        );
    }

    #[test]
    fn load_config_happy_case_default_profile() {
        let tempdir = copy_config_to_tempdir("tests/data/stubtool_snfoundry.toml", None);
        let config = load_config::<StubConfig>(
            Some(&Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap()),
            None,
        )
        .unwrap();
        assert_eq!(config.account, String::from("user1"));
        assert_eq!(
            config.url,
            Some(Url::parse("http://127.0.0.1:5055/rpc").unwrap())
        );
    }
    #[test]
    fn load_config_invalid_url() {
        let tempdir = copy_config_to_tempdir("tests/data/stubtool_snfoundry.toml", None);
        let err = load_config::<StubConfig>(
            Some(&Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap()),
            Some(&String::from("profile6")),
        )
        .unwrap_err();

        assert!(
            err.to_string()
                .contains("relative URL without a base: \"invalid_url\"")
        );
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
        assert_eq!(config.url, None);
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
            Ok(serde_json::from_value::<Self>(config)?)
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
    #[expect(clippy::float_cmp)]
    fn resolve_env_vars() {
        let tempdir =
            copy_config_to_tempdir("tests/data/stubtool_snfoundry.toml", Some("childdir1"));
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

        // Present env variables

        // SAFETY: These values are only read here and are not modified by other tests.
        unsafe {
            env::set_var("VALUE_STRING123132", "nfsaufbnsailfbsbksdabfnkl");
            env::set_var("VALUE_STRING123142", "nfsasnsidnnsailfbsbksdabdkdkl");
            env::set_var("VALUE_INT123132", "321312");
            env::set_var("VALUE_FLOAT123132", "321.312");
            env::set_var("VALUE_BOOL1231321", "true");
            env::set_var("VALUE_BOOL1231322", "false");
        };
        let config = load_config::<StubComplexConfig>(
            Some(&Utf8PathBuf::try_from(tempdir.path().to_path_buf()).unwrap()),
            Some(&String::from("with-envs")),
        )
        .unwrap();
        assert_eq!(config.url, String::from("nfsaufbnsailfbsbksdabfnkl"));
        assert_eq!(config.account, 321_312);
        assert_eq!(config.nested.list_example, vec![true, false]);
        assert_eq!(config.nested.url_nested, 321.312);
        assert_eq!(
            config.nested.url_alt,
            String::from("nfsasnsidnnsailfbsbksdabdkdkl")
        );
    }
}
