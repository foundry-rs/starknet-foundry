use anyhow::{Context, Result, bail};
use camino::Utf8Path;
use conversions::string::IntoHexStr;
use std::process::ExitCode;

use clap::Args;
use configuration::resolve_config_file;
use sncast::{
    helpers::{config::resolve_global_config_path_or_warn, configuration::CastConfig},
    response::ui::UI,
};
use starknet_types_core::felt::Felt;
use std::fs;
use toml_edit::{DocumentMut, Item, Table, Value};

#[derive(Args, Debug)]
pub struct Add {
    /// Name of the alias
    pub name: String,

    /// Felt value to associate with the alias
    pub value: Felt,

    /// Add the alias to the global configuration file
    #[arg(long)]
    pub global: bool,

    /// Overwrite an existing alias
    #[arg(long)]
    pub force: bool,
}

#[expect(clippy::needless_pass_by_value)]
pub fn add(args: Add, _config: &CastConfig, ui: &UI) -> Result<ExitCode> {
    let config_path = if args.global {
        resolve_global_config_path_or_warn(ui)
            .context("Failed to resolve the global configuration file")?
    } else {
        resolve_config_file()
    };

    let mut config = open_config(&config_path)?;
    add_alias(&mut config, &args.name, args.value, args.force)?;
    fs::write(config_path, config.to_string()).context("Failed to write to config file")?;

    Ok(ExitCode::SUCCESS)
}

fn open_config(path: &Utf8Path) -> Result<DocumentMut> {
    fs::read_to_string(path)
        .context("Failed to read config file")?
        .parse::<DocumentMut>()
        .context("Failed to parse TOML")
}

fn add_alias(config: &mut DocumentMut, name: &str, value: Felt, force: bool) -> Result<()> {
    let sncast_table = config
        .entry("sncast")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .context("Failed to create or access 'sncast' table")?;
    sncast_table.set_implicit(true);

    let default_profile = sncast_table
        .entry("default")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .context("Failed to create or access default profile table")?;

    let aliases_table = default_profile
        .entry("aliases")
        .or_insert(Item::Table(Table::new()))
        .as_table_mut()
        .context("Failed to create or access aliases table")?;

    if aliases_table.contains_key(name) && !force {
        bail!("Alias `{name}` already exists. Use --force to overwrite it.");
    }

    aliases_table[name] = Value::from(value.into_hex_string()).into();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn adds_alias_without_parent_fields() {
        let mut config = indoc! {r"
            [sncast.default]
        "}
        .parse::<DocumentMut>()
        .unwrap();

        add_alias(&mut config, "map", Felt::from(1), false).unwrap();

        assert_eq!(
            config.to_string(),
            indoc! {r#"
                [sncast.default]

                [sncast.default.aliases]
                map = "0x1"
            "#}
        );
    }

    #[test]
    fn adds_alias_with_parent_fields() {
        let mut config = indoc! {r#"
            [sncast.default]
            url = "http://127.0.0.1:5055/rpc"
        "#}
        .parse::<DocumentMut>()
        .unwrap();

        add_alias(&mut config, "map", Felt::from(1), false).unwrap();

        assert_eq!(
            config.to_string(),
            indoc! {r#"
                [sncast.default]
                url = "http://127.0.0.1:5055/rpc"

                [sncast.default.aliases]
                map = "0x1"
            "#}
        );
    }

    #[test]
    fn preserves_unrelated_fields_and_their_order() {
        let mut config = indoc! {r#"
            [snfoundry]
            exit-first = true

            [sncast.default]
            url = "http://127.0.0.1:5055/rpc"
            account = "devnet"

            [sncast.default.aliases]
            first = "0x1"
            third = "0x3"

            [snforge]
            max-n-steps = 123
        "#}
        .parse::<DocumentMut>()
        .unwrap();

        add_alias(&mut config, "second", Felt::from(2), false).unwrap();

        assert_eq!(
            config.to_string(),
            indoc! {r#"
                [snfoundry]
                exit-first = true

                [sncast.default]
                url = "http://127.0.0.1:5055/rpc"
                account = "devnet"

                [sncast.default.aliases]
                first = "0x1"
                third = "0x3"
                second = "0x2"

                [snforge]
                max-n-steps = 123
            "#}
        );
    }

    #[test]
    fn adds_alias_to_empty_config() {
        let mut config = DocumentMut::new();

        add_alias(&mut config, "map", Felt::from(1), false).unwrap();

        assert_eq!(
            config.to_string(),
            indoc! {r#"
                [sncast.default]

                [sncast.default.aliases]
                map = "0x1"
            "#}
        );
    }

    #[test]
    fn errors_when_alias_already_exists_without_force() {
        let mut config = indoc! {r#"
            [sncast.default.aliases]
            map = "0x1"
        "#}
        .parse::<DocumentMut>()
        .unwrap();

        let error = add_alias(&mut config, "map", Felt::from(2), false).unwrap_err();

        assert_eq!(
            error.to_string(),
            "Alias `map` already exists. Use --force to overwrite it."
        );
        assert_eq!(
            config.to_string(),
            indoc! {r#"
                [sncast.default.aliases]
                map = "0x1"
            "#}
        );
    }

    #[test]
    fn replaces_existing_alias_with_force() {
        let mut config = indoc! {r#"
            [sncast.default.aliases]
            map = "0x1"
        "#}
        .parse::<DocumentMut>()
        .unwrap();

        add_alias(&mut config, "map", Felt::from(2), true).unwrap();

        assert_eq!(
            config.to_string(),
            indoc! {r#"
                [sncast.default.aliases]
                map = "0x2"
            "#}
        );
    }
}
