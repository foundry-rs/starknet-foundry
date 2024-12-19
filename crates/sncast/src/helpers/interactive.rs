use crate::helpers::config::get_global_config_path;
use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use configuration::search_config_upwards_relative_to;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use std::env::current_dir;
use std::fs;
use toml_edit::{DocumentMut, Item, Table, Value};

pub fn prompt_to_add_account_as_default(account: &str) -> Result<()> {
    let mut options = Vec::new();

    if let Some(option) = get_global_config_path().ok().map(|global_path| {
        format!(
            "Yes, global default account ({}).",
            to_tilde_path(&global_path)
        )
    }) {
        options.push(option);
    }

    if let Some(option) = current_dir()
        .ok()
        .and_then(|current_path| Utf8PathBuf::from_path_buf(current_path.clone()).ok())
        .and_then(|current_path_utf8| search_config_upwards_relative_to(&current_path_utf8).ok())
        .map(|local_path| {
            format!(
                "Yes, local default account ({}).",
                to_tilde_path(&local_path)
            )
        })
    {
        options.push(option);
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to make this account default?")
        .items(&options)
        .default(0)
        .interact()
        .context("Failed to display selection dialog")?;

    match options[selection].as_str() {
        selected if selected.starts_with("Yes, global default") => {
            if let Ok(global_path) = get_global_config_path() {
                edit_config(&global_path, "default", "account", account)?;
            }
        }
        selected if selected.starts_with("Yes, local default") => {
            if let Ok(current_path) = current_dir() {
                let current_path_utf8 = Utf8PathBuf::from_path_buf(current_path).map_err(|_| {
                    anyhow!("Failed to convert current directory path to Utf8PathBuf")
                })?;

                if let Ok(local_path) = search_config_upwards_relative_to(&current_path_utf8) {
                    edit_config(&local_path, "default", "account", account)?;
                }
            }
        }
        _ => {}
    }

    Ok(())
}

fn edit_config(config_path: &Utf8PathBuf, profile: &str, key: &str, value: &str) -> Result<()> {
    let file_content = fs::read_to_string(config_path)?;

    let mut toml_doc = file_content
        .parse::<DocumentMut>()
        .context("Failed to parse TOML")?;
    update_config(&mut toml_doc, profile, key, value);

    fs::write(config_path, toml_doc.to_string())?;

    Ok(())
}

fn update_config(toml_doc: &mut DocumentMut, profile: &str, key: &str, value: &str) {
    if !toml_doc.contains_key("sncast") {
        toml_doc["sncast"] = Item::Table(Table::new());
    }

    let sncast_table = toml_doc
        .get_mut("sncast")
        .and_then(|item| item.as_table_mut())
        .expect("Failed to create or access 'sncast' table");

    if !sncast_table.contains_key(profile) {
        sncast_table[profile] = Item::Table(Table::new());
    }

    let profile_table = sncast_table
        .get_mut(profile)
        .and_then(|item| item.as_table_mut())
        .expect("Failed to create or access profile table");

    profile_table[key] = Value::from(value).into();
}

fn to_tilde_path(path: &Utf8PathBuf) -> String {
    if cfg!(not(target_os = "windows")) {
        if let Some(home_dir) = dirs::home_dir() {
            if let Ok(stripped_path) = path.strip_prefix(&home_dir) {
                return format!("~/{stripped_path}");
            }
        }
    }

    path.to_string()
}

#[cfg(test)]
mod tests {

    use super::update_config;
    use indoc::formatdoc;
    use toml_edit::DocumentMut;
    #[test]
    fn test_update_value() {
        let original = formatdoc! {r#"
            [snfoundry]
            key = 2137

            [sncast.default]
            account = "mainnet"
            url = "https://localhost:5050"

            # comment

            [sncast.testnet]
            account = "testnet-account"        # comment
            url = "https://swmansion.com/"
        "#};

        let expected = formatdoc! {r#"
            [snfoundry]
            key = 2137

            [sncast.default]
            account = "testnet"
            url = "https://localhost:5050"

            # comment

            [sncast.testnet]
            account = "testnet-account"        # comment
            url = "https://swmansion.com/"
        "#};

        let mut toml_doc = original.parse::<DocumentMut>().unwrap();

        update_config(&mut toml_doc, "default", "account", "testnet");

        assert_eq!(toml_doc.to_string(), expected);
    }

    #[test]
    fn test_create_key() {
        let original = formatdoc! {r#"
            [snfoundry]
            key = 2137

            [sncast.default]
            url = "https://localhost:5050"

            [sncast.testnet]
            account = "testnet-account"        # comment
            url = "https://swmansion.com/"
        "#};

        let expected = formatdoc! {r#"
            [snfoundry]
            key = 2137

            [sncast.default]
            url = "https://localhost:5050"
            account = "testnet"

            [sncast.testnet]
            account = "testnet-account"        # comment
            url = "https://swmansion.com/"
        "#};

        let mut toml_doc = original.parse::<DocumentMut>().unwrap();

        update_config(&mut toml_doc, "default", "account", "testnet");

        assert_eq!(toml_doc.to_string(), expected);
    }
}
