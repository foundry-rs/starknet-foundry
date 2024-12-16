use crate::helpers::config::{create_global_config, get_global_config_path};
use anyhow::{Context, Result};
use camino::Utf8PathBuf;
use configuration::search_config_upwards_relative_to;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
use std::env::current_dir;
use std::fs::File;
use std::io::{Read, Write};
use toml_edit::{DocumentMut, Item, Table, Value};

pub fn ask_to_add_as_default(account: &str) -> Result<()> {
    let mut options = Vec::new();

    if let Ok(global_path) = get_global_config_path() {
        let option = format!("Yes, global default account ({global_path}).");
        options.push(option);
    }

    if let Ok(current_path) = current_dir() {
        let current_path_utf8 = Utf8PathBuf::from_path_buf(current_path.clone())
            .expect("Failed to convert current directory to Utf8PathBuf");

        if let Ok(local_path) = search_config_upwards_relative_to(&current_path_utf8) {
            let option = format!("Yes, local default account ({local_path}).");
            options.push(option);
        } else {
            let new_local_config_path = current_path.join("snfoundry.toml");
            let option = format!(
                "Yes, create new local config with default account ({}).",
                new_local_config_path.display()
            );
            options.push(option);
        }
    }

    options.push("No".to_string());

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
                let current_path_utf8 = Utf8PathBuf::from_path_buf(current_path)
                    .expect("Failed to convert current directory to Utf8PathBuf");

                if let Ok(local_path) = search_config_upwards_relative_to(&current_path_utf8) {
                    edit_config(&local_path, "default", "account", account)?;
                }
            }
        }
        selected if selected.starts_with("Yes, create new") => {
            if let Ok(current_path) = current_dir() {
                let new_local_config_path = current_path.join("snfoundry.toml");
                let new_config_utf8 = Utf8PathBuf::from_path_buf(new_local_config_path)
                    .expect("Failed to convert new config path to Utf8PathBuf");

                create_global_config(new_config_utf8.clone())?;
                edit_config(&new_config_utf8, "default", "account", account)?;
            }
        }
        _ => {}
    }

    Ok(())
}

pub fn edit_config(config_path: &Utf8PathBuf, profile: &str, key: &str, value: &str) -> Result<()> {
    let mut file_content = String::new();
    File::open(config_path)?.read_to_string(&mut file_content)?;

    let mut toml_doc = file_content
        .parse::<DocumentMut>()
        .expect("Failed to parse TOML");
    update_config(&mut toml_doc, profile, key, value);

    File::create(config_path)?.write_all(toml_doc.to_string().as_bytes())?;

    Ok(())
}

pub fn update_config(toml_doc: &mut DocumentMut, profile: &str, key: &str, value: &str) {
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
