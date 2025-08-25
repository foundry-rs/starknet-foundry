mod coverage;

use crate::TestArgs;
use crate::profile_validation::coverage::check_coverage_compatibility;
use scarb_metadata::Metadata;
use std::fs;
use toml_edit::{DocumentMut, Table};

/// Checks if current profile provided in [`Metadata`] can be used to run coverage and backtrace if applicable.
pub fn check_profile_compatibility(
    test_args: &TestArgs,
    scarb_metadata: &Metadata,
) -> anyhow::Result<()> {
    if test_args.coverage {
        check_coverage_compatibility(scarb_metadata)?;
    }
    Ok(())
}

/// Gets the runtime manifest from the [`Metadata`] and parses it into a [`DocumentMut`].
fn get_manifest(scarb_metadata: &Metadata) -> anyhow::Result<DocumentMut> {
    Ok(fs::read_to_string(&scarb_metadata.runtime_manifest)?.parse::<DocumentMut>()?)
}

/// Check if the Cairo profile entries in the manifest contain the required entries.
fn check_cairo_profile_entries(
    manifest: &DocumentMut,
    scarb_metadata: &Metadata,
    required_entries: &[(&str, &str)],
) -> bool {
    manifest
        .get("profile")
        .and_then(|profile| profile.get(&scarb_metadata.current_profile))
        .and_then(|profile| profile.get("cairo"))
        .and_then(|cairo| cairo.as_table())
        .is_some_and(|profile_cairo| {
            required_entries
                .iter()
                .all(|(key, value)| contains_entry_with_value(profile_cairo, key, value))
        })
}

/// Check if the table contains an entry with the given key and value.
/// Accepts only bool and string values.
fn contains_entry_with_value(table: &Table, key: &str, value: &str) -> bool {
    table.get(key).is_some_and(|entry| {
        if let Some(entry) = entry.as_bool() {
            entry.to_string() == value
        } else if let Some(entry) = entry.as_str() {
            entry == value
        } else {
            false
        }
    })
}
