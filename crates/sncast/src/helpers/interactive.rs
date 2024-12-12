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

    if let Some(global_path) = get_global_config_path().ok() {
        let option = format!("Yes, global default account ({}).", global_path);
        options.push(option);
    }

    if let Some(current_path) = current_dir().ok() {
        let current_path_utf8 = Utf8PathBuf::from_path_buf(current_path.clone())
            .expect("Failed to convert current directory to Utf8PathBuf");

        if let Some(local_path) = search_config_upwards_relative_to(&current_path_utf8).ok() {
            let option = format!("Yes, local default account ({}).", local_path);
            options.push(option);
        } else {
            let new_local_config_path = current_path.join("snfoundry.toml");
            let option = format!(
                "Yes, create new local default account ({}).",
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
        selected if selected.starts_with("Yes, global default account") => {
            if let Some(global_path) = get_global_config_path().ok() {
                edit_config(global_path, "default", "account", account)?;
            }
        }
        selected if selected.starts_with("Yes, local default account") => {
            if let Some(current_path) = current_dir().ok() {
                let current_path_utf8 = Utf8PathBuf::from_path_buf(current_path)
                    .expect("Failed to convert current directory to Utf8PathBuf");

                if let Some(local_path) = search_config_upwards_relative_to(&current_path_utf8).ok()
                {
                    edit_config(local_path, "default", "account", account)?;
                }
            }
        }
        selected if selected.starts_with("Yes, create new local default account") => {
            if let Some(current_path) = current_dir().ok() {
                let new_local_config_path = current_path.join("snfoundry.toml");
                let new_config_utf8 = Utf8PathBuf::from_path_buf(new_local_config_path)
                    .expect("Failed to convert new config path to Utf8PathBuf");

                create_global_config(new_config_utf8.clone())?;
                edit_config(new_config_utf8, "default", "account", account)?;
            }
        }
        _ => {
            println!("No changes were made.");
        }
    }

    Ok(())
}

pub fn edit_config(config_path: Utf8PathBuf, profile: &str, key: &str, value: &str) -> Result<()> {
    let mut file_content = String::new();
    File::open(&config_path)?.read_to_string(&mut file_content)?;

    let mut toml_doc = file_content
        .parse::<DocumentMut>()
        .expect("Failed to parse TOML")?;
    update_config(&mut toml_doc, profile, key, value);

    File::create(&config_path)?.write_all(toml_doc.to_string().as_bytes())?;

    Ok(())
}

pub fn update_config(toml_doc: &mut DocumentMut, profile: &str, key: &str, value: &str) {
    if let Some(sncast_table) = toml_doc
        .get_mut("sncast")
        .and_then(|item| item.as_table_mut())
    {
        if !sncast_table.contains_key(profile) {
            let mut profile_table = Table::new();
            profile_table[key] = Value::from(value).into();
            sncast_table[profile] = Item::Table(profile_table);
        } else {
            let profile_table = sncast_table
                .get_mut(profile)
                .unwrap()
                .as_table_mut()
                .unwrap();

            profile_table[key] = Value::from(value).into();
        }
    } else {
        let mut profile_table = Table::new();
        profile_table[key] = Value::from(value).into();

        let mut sncast_table = Table::new();
        sncast_table[profile] = Item::Table(profile_table);
        toml_doc["sncast"] = Item::Table(sncast_table);
    }
}
