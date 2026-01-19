use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use std::fs;
use toml_edit::{DocumentMut, Item, value};

pub struct ManifestEditor {
    path: Utf8PathBuf,
}

impl ManifestEditor {
    pub fn new(manifest_path: &Utf8Path) -> Self {
        Self {
            path: manifest_path.to_owned(),
        }
    }

    pub fn set_inlining_strategy(&self, threshold: u32, profile: &str) -> Result<()> {
        let content = fs::read_to_string(&self.path)?;
        let mut doc = content
            .parse::<DocumentMut>()
            .context("Failed to parse Scarb.toml")?;

        let profile_table = doc
            .entry("profile")
            .or_insert(toml_edit::Item::Table(toml_edit::Table::new()))
            .as_table_mut()
            .context("Invalid profile section")?;

        let profile_entry = profile_table
            .entry(profile)
            .or_insert(toml_edit::Item::Table(toml_edit::Table::new()))
            .as_table_mut()
            .context("Invalid profile entry")?;

        let cairo_table = profile_entry
            .entry("cairo")
            .or_insert(toml_edit::Item::Table(toml_edit::Table::new()))
            .as_table_mut()
            .context("Invalid cairo section")?;

        cairo_table["inlining-strategy"] = value(i64::from(threshold));

        fs::write(&self.path, doc.to_string())?;
        Ok(())
    }
}

pub(crate) fn overwrite_starknet_contract_target_flags(doc: &mut DocumentMut) -> bool {
    let Some(target_item) = doc.as_table_mut().get_mut("target") else {
        return false;
    };
    let Item::Table(target_table) = target_item else {
        return false;
    };
    let Some(starknet_contract_item) = target_table.get_mut("starknet-contract") else {
        return false;
    };
    let Item::ArrayOfTables(starknet_contract_tables) = starknet_contract_item else {
        return false;
    };

    let mut changed = false;
    for target in starknet_contract_tables.iter_mut() {
        if target.get("casm").and_then(Item::as_bool) != Some(true) {
            target["casm"] = value(true);
            changed = true;
        }
        if target.get("sierra").and_then(Item::as_bool) != Some(true) {
            target["sierra"] = value(true);
            changed = true;
        }
    }

    changed
}
