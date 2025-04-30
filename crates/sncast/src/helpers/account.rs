use crate::NestedMap;
use anyhow::Result;
use camino::Utf8PathBuf;
use std::collections::HashSet;

use crate::{AccountData, read_and_parse_json_file};

pub fn generate_account_name(accounts_file: &Utf8PathBuf) -> Result<String> {
    let mut id = 1;

    if !accounts_file.exists() {
        return Ok(format!("account-{id}"));
    }

    let networks: NestedMap<AccountData> = read_and_parse_json_file(accounts_file)?;
    let mut result = HashSet::new();

    for (_, accounts) in networks {
        for (name, _) in accounts {
            if let Some(id) = name
                .strip_prefix("account-")
                .and_then(|id| id.parse::<u32>().ok())
            {
                result.insert(id);
            }
        }
    }

    while result.contains(&id) {
        id += 1;
    }

    Ok(format!("account-{id}"))
}
