use crate::helpers::configuration::{show_explorer_links_default, CastConfig};
use crate::ValidatedWaitParams;
use anyhow::Result;
use camino::Utf8PathBuf;
use indoc::formatdoc;
use shared::consts::FREE_RPC_PROVIDER_URL;
use std::fs;
use std::fs::File;
use std::io::Write;

pub fn get_global_config_path() -> Result<Utf8PathBuf> {
    let global_config_dir = {
        if cfg!(target_os = "windows") {
            dirs::config_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
                .join("starknet-foundry")
        } else {
            dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?
                .join(".config/starknet-foundry")
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
        # url = "{default_url}"
        # block-explorer = "{default_block_explorer}"
        # wait-params = {{ timeout = {default_wait_timeout}, retry-interval = {default_wait_retry_interval} }}
        # show-explorer-links = {default_show_explorer_links}
        # accounts-file = "{default_accounts_file}"
        # account = "{default_account}"
        # keystore = "{default_keystore}"
        "#,
        default_url = FREE_RPC_PROVIDER_URL,
        default_accounts_file = "~/.starknet_accounts/starknet_open_zeppelin_accounts.json",
        default_wait_timeout = default_wait_params.timeout,
        default_wait_retry_interval = default_wait_params.retry_interval,
        default_block_explorer = "StarkScan",
        default_show_explorer_links = show_explorer_links_default(),
        default_account = "default",
        default_keystore = ""
    }
}

fn create_global_config(global_config_path: Utf8PathBuf) -> Result<()> {
    let mut file = File::create(global_config_path)?;
    file.write_all(build_default_manifest().as_bytes())?;

    Ok(())
}
macro_rules! clone_field {
    ($global_config:expr, $local_config:expr, $default_config:expr, $field:ident) => {
        if $local_config.$field != $default_config.$field {
            $local_config.$field.clone()
        } else {
            $global_config.$field.clone()
        }
    };
}

#[must_use]
pub fn combine_cast_configs(global_config: &CastConfig, local_config: &CastConfig) -> CastConfig {
    let default_cast_config = CastConfig::default();

    CastConfig {
        url: clone_field!(global_config, local_config, default_cast_config, url),
        account: clone_field!(global_config, local_config, default_cast_config, account),
        accounts_file: clone_field!(
            global_config,
            local_config,
            default_cast_config,
            accounts_file
        ),
        keystore: clone_field!(global_config, local_config, default_cast_config, keystore),
        wait_params: clone_field!(
            global_config,
            local_config,
            default_cast_config,
            wait_params
        ),
        block_explorer: clone_field!(
            global_config,
            local_config,
            default_cast_config,
            block_explorer
        ),
        show_explorer_links: clone_field!(
            global_config,
            local_config,
            default_cast_config,
            show_explorer_links
        ),
    }
}
