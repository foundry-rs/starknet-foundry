use crate::helpers::constants::STATE_FILE_VERSION;
use anyhow::{anyhow, Result};
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Serialize, Debug)]
pub struct StateFile {
    pub version: u8,
    pub transactions: Option<ScriptTransactionEntries>,
}

impl StateFile {
    pub fn append_transaction_entries(&mut self, tx_entries: ScriptTransactionEntries) {
        match self.transactions {
            Some(ref mut existing_entries) => {
                existing_entries.transactions.extend(tx_entries.transactions.into_iter());
            },
            None => {
                self.transactions = Some(tx_entries);
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct ScriptTransactionEntries {
    pub transactions: HashMap<String, ScriptTransactionEntry>, // todo: get rid of hashmaps
}

impl ScriptTransactionEntries {
    pub fn get(&self, key: &str) -> Option<&ScriptTransactionEntry> {
        self.transactions.get(key)
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct ScriptTransactionEntry {
    pub name: String,
    pub output: ScriptTransactionOutput,
    pub status: ScriptTransactionStatus,
    pub timestamp: u32,
    pub misc: Option<HashMap<String, Value>>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct ScriptTransactionOutput {
    #[serde(flatten)]
    pub output: HashMap<String, Value>, // todo: get rid of hashmaps
}

impl ScriptTransactionOutput {
    pub fn get(&self, key: &str) -> Result<&Value, &'static str> {
        self.output.get(key).ok_or("Key not found in outputs struct")
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub enum ScriptTransactionStatus {
    // script executed successfully, transaction accepted/succeeded
    Success,
    // script executed successfully, transaction not rejected/reverted
    Fail,
    // script error
    Error,
}

pub fn load_or_create_state_file(path: &Utf8PathBuf) -> Result<StateFile> {
    if path.exists() {
        let content = fs::read_to_string(path).expect("Failed to read state file");
        match serde_json::from_str::<StateFile>(&content) {
            Ok(state_file) => Ok(state_file),
            Err(_) => Err(anyhow!("Failed to parse state file - it may be corrupt")),
        }
    } else {
        let default_state = StateFile {
            version: STATE_FILE_VERSION,
            transactions: None,
        };
        fs::write(
            &path,
            serde_json::to_string_pretty(&default_state)
                .expect("Failed to convert StateFile to json"),
        )
        .expect("Failed to write initial state to state file");
        Ok(default_state)
    }
}

// pub fn generate_transaction_entry(tx_entry: ScriptTransactionEntry) -> ScriptTransactionEntries {
//     // todo (1545): Implement hashing function for unique ids
//     let id = format!("{}-{}", tx_entry.name, tx_entry.timestamp);
//     ScriptTransactionEntries
// }

pub fn write_txs_to_state_file(state_file_path: &Utf8PathBuf, tx_entries: ScriptTransactionEntries) -> Result<()> {
    let mut state_file = load_or_create_state_file(state_file_path).expect("Failed to load state file");
    state_file.append_transaction_entries(tx_entries);
    Ok(())
}

// sprawdzanie wersji
// zapisywanie do pliku przy użyciu pliku .bcp
// pozwolić userom zapisywać coś do misc

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_load_or_create_state_file_new_happy() {
        let tempdir = TempDir::new().unwrap();
        let state_file = Utf8PathBuf::from_path_buf(
            tempdir
                .path()
                .join("test_load_or_create_state_file_new_happy.json")
                .into(),
        )
        .unwrap();

        let result = load_or_create_state_file(&state_file).unwrap();

        assert_eq!(result.version, 1);
        assert_eq!(result.transactions, None);
    }

    #[test]
    fn test_load_or_create_state_file_exists_no_txs() {
        let state_file = Utf8PathBuf::from("tests/data/files/state_no_txs.json");
        let result = load_or_create_state_file(&state_file).unwrap();

        assert_eq!(result.version, 1);
        assert_eq!(result.transactions, None);
    }

    #[test]
    fn test_load_or_create_state_file_exists_with_tx() {
        let state_file = Utf8PathBuf::from("tests/data/files/state_with_tx.json");
        let result = load_or_create_state_file(&state_file).unwrap();

        assert_eq!(result.transactions.unwrap().get("123-abc-789").unwrap().name, "declare");
    }

    #[test]
    fn test_load_or_create_state_file_exists_with_txs() {
        let state_file = Utf8PathBuf::from("tests/data/files/state_with_txs.json");
        let result = load_or_create_state_file(&state_file).unwrap();

        assert_eq!(result.transactions.unwrap().get("789-def-420").unwrap().output.get("error").unwrap().as_str().unwrap(), "Max fee is smaller than the minimal transaction cost");
    }

    #[test]
    fn test_load_or_create_state_file_exists_corrupt() {
        let state_file = Utf8PathBuf::from("tests/data/files/state_corrupt_missing_field.json");
        let result = load_or_create_state_file(&state_file).unwrap_err();
        assert_eq!(result.to_string(), "Failed to parse state file - it may be corrupt");
    }

    // #[test]
    // fn test_write_to_file_no_file() {
    //     let tempdir = TempDir::new().unwrap();
    //     let state_file_path = Utf8PathBuf::from_path_buf(
    //         tempdir
    //             .path()
    //             .join("write_state_nofile.json")
    //             .into(),
    //     )
    //     .unwrap();
    //
    //     HashMap.
    //     let result = write_txs_to_state_file(&state_file_path).unwrap();
    // }

    // #[test]
    // fn test_write_to_file_txs_existed() {
    //
    // }

    // #[test]
    // fn test_write_to_file_wrong_entries???() {
    //
    // }

    // #[test]
    // fn test_load_or_create_state_file_exists_version_mismatch() {
    //
    // }
}
