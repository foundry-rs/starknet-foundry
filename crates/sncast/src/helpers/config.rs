use crate::ValidatedWaitParams;
use crate::helpers::configuration::show_explorer_links_default;
use crate::helpers::constants::DEFAULT_ACCOUNTS_FILE;
use anyhow::Result;
use camino::Utf8PathBuf;
use indoc::formatdoc;
use std::fs::File;
use std::io::Write;
use std::{env, fs};

pub fn get_or_create_global_config_path() -> Result<Utf8PathBuf> {
    let global_config_dir = match env::var("SNFOUNDRY_CONFIG").ok() {
        Some(dir) => Utf8PathBuf::from(shellexpand::tilde(&dir).to_string()).into_std_path_buf(),
        None => {
            if cfg!(target_os = "windows") {
                dirs::config_dir()
                    .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
                    .join("starknet-foundry")
            } else {
                dirs::home_dir()
                    .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
                    .join(".config/starknet-foundry")
            }
        }
    };

    if !global_config_dir.exists() {
        fs::create_dir_all(&global_config_dir)?;
    }

    let global_config_path = Utf8PathBuf::from_path_buf(global_config_dir.join("snfoundry.toml"))
        .expect("Failed to convert PathBuf to Utf8PathBuf for global configuration");

    if !global_config_path.exists() {
        create_global_config(global_config_path.clone())?;
    }

    Ok(global_config_path)
}

fn build_default_manifest() -> String {
    let default_wait_params = ValidatedWaitParams::default();

    formatdoc! {r#"
        # Visit https://foundry-rs.github.io/starknet-foundry/appendix/snfoundry-toml.html
        # and https://foundry-rs.github.io/starknet-foundry/projects/configuration.html for more information

        # [sncast.default]
        # url = ""
        # block-explorer = "{default_block_explorer}"
        # wait-params = {{ timeout = {default_wait_timeout}, retry-interval = {default_wait_retry_interval} }}
        # show-explorer-links = {default_show_explorer_links}
        # accounts-file = "{default_accounts_file}"
        # account = ""
        # keystore = ""
        
        # Configure custom network addresses
        # [sncast.default.networks]
        # mainnet = "https://mainnet.your-node.com"
        # sepolia = "https://sepolia.your-node.com"
        # devnet = "http://127.0.0.1:5050/rpc"
        "#,
        default_accounts_file = DEFAULT_ACCOUNTS_FILE,
        default_wait_timeout = default_wait_params.get_timeout(),
        default_wait_retry_interval = default_wait_params.get_retry_interval(),
        default_block_explorer = "Voyager",
        default_show_explorer_links = show_explorer_links_default(),
    }
}

fn create_global_config(global_config_path: Utf8PathBuf) -> Result<()> {
    let mut file = File::create(global_config_path)?;
    file.write_all(build_default_manifest().as_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::configuration::PartialCastConfig;
    use configuration::load_config;
    use tempfile::tempdir;

    #[test]
    fn build_default_manifest_produces_valid_config_when_uncommented() {
        let manifest = build_default_manifest();
        let lines: Vec<&str> = manifest.lines().collect();
        assert_eq!(
            lines[0],
            "# Visit https://foundry-rs.github.io/starknet-foundry/appendix/snfoundry-toml.html"
        );
        assert_eq!(
            lines[1],
            "# and https://foundry-rs.github.io/starknet-foundry/projects/configuration.html for more information"
        );

        let toml: String = lines
            .into_iter()
            .filter_map(|l| l.strip_prefix("# "))
            // skip comments that are not part of the config
            .filter(|l| l.starts_with('[') || l.contains('='))
            // drop `url = ""`
            .filter(|l| *l != "url = \"\"")
            .collect::<Vec<_>>()
            .join("\n");

        let t = tempdir().unwrap();
        let path = Utf8PathBuf::try_from(t.path().join("snfoundry.toml")).unwrap();
        fs::write(&path, toml).unwrap();

        let config: PartialCastConfig = load_config(&path, None).unwrap().unwrap();
        config.validate().unwrap();
    }
}
