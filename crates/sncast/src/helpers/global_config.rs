use crate::helpers::configuration::show_explorer_links_default;
use crate::ValidatedWaitParams;
use anyhow::Result;
use camino::Utf8PathBuf;
use indoc::formatdoc;
use std::fs;
use std::fs::File;
use std::io::Write;

pub fn get_global_config_path() -> Result<Utf8PathBuf> {
    let config_dir = if cfg!(target_os = "windows") {
        dirs::config_dir()
    } else {
        dirs::home_dir()
    }
    .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    let global_config_dir = config_dir.join(if cfg!(target_os = "windows") {
        "starknet-foundry"
    } else {
        ".config/starknet-foundry"
    });

    if !global_config_dir.exists() {
        fs::create_dir_all(&global_config_dir)?;
    }

    let global_config_path =
        Utf8PathBuf::from_path_buf(global_config_dir.join("snfoundry.toml")).unwrap();

    if global_config_path.exists() {
        Ok(global_config_path)
    } else {
        create_global_config(global_config_path.clone())?;
        Ok(global_config_path)
    }
}

fn build_default_config() -> String {
    let default_url = "https://starknet-sepolia.public.blastapi.io";
    let default_accounts_file = "~/.starknet_accounts/starknet_open_zeppelin_accounts.json";
    let default_wait_params = ValidatedWaitParams::default();
    let default_wait_timeout = default_wait_params.timeout;
    let default_wait_retry_interval = default_wait_params.retry_interval;
    let default_block_explorer = "StarkScan";
    let default_show_explorer_links = show_explorer_links_default();
    let default_account = "default";
    let default_keystore = "";

    formatdoc! {r#"
        # Visit https://foundry-rs.github.io/starknet-foundry/appendix/snfoundry-toml.html
        # and https://foundry-rs.github.io/starknet-foundry/projects/configuration.html for more information

        [sncast.default]
        url = "{default_url}"
        accounts-file = "{default_accounts_file}"
        block-explorer = "{default_block_explorer}"
        wait-params = {{ timeout = {default_wait_timeout}, retry-interval = {default_wait_retry_interval} }}
        show-explorer-links = {default_show_explorer_links}
        account = "{default_account}"
        keystore = "{default_keystore}"
        "#
    }
}

fn create_global_config(global_config_path: Utf8PathBuf) -> Result<()> {
    let mut file = File::create(global_config_path)?;
    file.write_all(build_default_config().as_bytes())?;

    Ok(())
}
