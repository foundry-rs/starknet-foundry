use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use std::fs;
use toml_edit::{DocumentMut, value};

pub struct ManifestEditor {
    path: Utf8PathBuf,
}

impl ManifestEditor {
    pub fn new(manifest_path: &Utf8Path) -> Result<Self> {
        Ok(Self {
            path: manifest_path.to_owned(),
        })
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

        cairo_table["inlining-strategy"] = value(threshold as i64);

        fs::write(&self.path, doc.to_string())?;
        Ok(())
    }
}
