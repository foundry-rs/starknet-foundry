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

fn create_global_config(global_config_path: Utf8PathBuf) -> Result<()> {
    let mut file = File::create(global_config_path)?;
    file.write_all(
        formatdoc! {r#"
        # Visit https://foundry-rs.github.io/starknet-foundry/appendix/snfoundry-toml.html
        # and https://foundry-rs.github.io/starknet-foundry/projects/configuration.html for more information

        [sncast.default]
        url = "https://starknet-sepolia.public.blastapi.io"
        # accounts-file = "~/.starknet_accounts/starknet_open_zeppelin_accounts.json"
        # wait-params = {{ timeout = 500, retry-interval = 10 }}
        # block-explorer = "StarkScan"
        # show-explorer-links = true
        # account = "default"
        # keystore = ""
        "#
        }.as_bytes(),
    )?;

    Ok(())
}
