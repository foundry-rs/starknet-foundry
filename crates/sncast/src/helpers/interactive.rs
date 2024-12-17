use crate::helpers::config::get_global_config_path;
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use configuration::search_config_upwards_relative_to;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use std::env::current_dir;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use toml_edit::{DocumentMut, Item, Table, Value};

enum PromptSelection {
    GlobalDefault(String, Utf8PathBuf),
    LocalDefault(String, Utf8PathBuf),
    No,
}

impl Display for PromptSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            PromptSelection::LocalDefault(s, _) | PromptSelection::GlobalDefault(s, _) => {
                s.to_string()
            }
            PromptSelection::No => "No".to_string(),
        };
        write!(f, "{str}")
    }
}

pub fn prompt_to_add_account_as_default(account: &str) -> Result<()> {
    let mut options = Vec::new();

    if let Ok(global_path) = get_global_config_path() {
        options.push(PromptSelection::GlobalDefault(
            format!(
                "Yes, global default ({})",
                to_tilde_path(global_path.as_str())
            ),
            global_path.clone(),
        ));
    }

    if let Some(local_path) = current_dir()
        .ok()
        .and_then(|current_path| Utf8PathBuf::from_path_buf(current_path.clone()).ok())
        .and_then(|current_path_utf8| search_config_upwards_relative_to(&current_path_utf8).ok())
    {
        options.push(PromptSelection::LocalDefault(
            format!(
                "Yes, local default ({})",
                to_tilde_path(local_path.as_str())
            ),
            local_path.clone(),
        ));
    }

    options.push(PromptSelection::No);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to make this account default?")
        .items(&options)
        .default(0)
        .interact()
        .context("Failed to display selection dialog")?;

    match &options[selection] {
        PromptSelection::GlobalDefault(_, global_path) => {
            edit_config(global_path, "default", "account", account)?;
        }
        PromptSelection::LocalDefault(_, local_path) => {
            edit_config(local_path, "default", "account", account)?;
        }
        PromptSelection::No => {}
    }

    Ok(())
}

fn edit_config(config_path: &Utf8PathBuf, profile: &str, key: &str, value: &str) -> Result<()> {
    let mut file_content = String::new();
    File::open(config_path)?.read_to_string(&mut file_content)?;

    let mut toml_doc = file_content
        .parse::<DocumentMut>()
        .context("Failed to parse TOML")?;
    update_config(&mut toml_doc, profile, key, value);

    File::create(config_path)?.write_all(toml_doc.to_string().as_bytes())?;

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

fn to_tilde_path(path: &str) -> String {
    if cfg!(not(target_os = "windows")) {
        if let Some(home_dir) = dirs::home_dir() {
            let input_path = Path::new(path);

            if let Ok(stripped_path) = input_path.strip_prefix(&home_dir) {
                return format!("~/{}", stripped_path.display());
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
