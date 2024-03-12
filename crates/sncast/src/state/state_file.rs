#![allow(dead_code)]
use crate::helpers::constants::STATE_FILE_VERSION;
use crate::response::structs::{DeclareResponse, DeployResponse, InvokeResponse};
use crate::state::hashing::generate_id;
use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Serialize, Debug)]
pub struct ScriptTransactionsSchema {
    pub version: u8,
    pub transactions: Option<ScriptTransactionEntries>,
}

impl ScriptTransactionsSchema {
    pub fn append_transaction_entries(&mut self, tx_entries: ScriptTransactionEntries) {
        match self.transactions {
            Some(ref mut existing_entries) => {
                existing_entries
                    .transactions
                    .extend(tx_entries.transactions);
            }
            None => {
                self.transactions = Some(tx_entries);
            }
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct ScriptTransactionEntries {
    pub transactions: HashMap<String, ScriptTransactionEntry>,
}

impl ScriptTransactionEntries {
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&ScriptTransactionEntry> {
        self.transactions.get(key)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct ScriptTransactionEntry {
    pub name: String,
    pub output: ScriptTransactionOutput,
    pub status: ScriptTransactionStatus,
    pub timestamp: u32,
    pub misc: Option<HashMap<String, Value>>,
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum ScriptTransactionOutput {
    InvokeResponse(InvokeResponse),
    DeclareResponse(DeclareResponse),
    DeployResponse(DeployResponse),
    ErrorResponse(ErrorResponse),
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub enum ScriptTransactionStatus {
    // script executed successfully, transaction accepted/succeeded
    Success,
    // script executed successfully, transaction rejected/reverted
    Fail,
    // script error
    Error,
}

pub fn load_state_file(path: &Utf8PathBuf) -> Result<ScriptTransactionsSchema> {
    let content = fs::read_to_string(path).context("Failed to load state file")?;
    match serde_json::from_str::<ScriptTransactionsSchema>(&content) {
        Ok(state_file) => {
            verify_version(state_file.version)?;
            Ok(state_file)
        }
        Err(_) => Err(anyhow!("Failed to parse state file - it may be corrupt")),
    }
}

pub fn load_or_create_state_file(path: &Utf8PathBuf) -> Result<ScriptTransactionsSchema> {
    if path.exists() {
        load_state_file(path)
    } else {
        let default_state = ScriptTransactionsSchema {
            version: STATE_FILE_VERSION,
            transactions: None,
        };
        fs::write(
            path,
            serde_json::to_string_pretty(&default_state)
                .expect("Failed to convert ScriptTransactionsSchema to json"),
        )
        .expect("Failed to write initial state to state file");
        Ok(default_state)
    }
}

// todo (1233): remove must_use attribute when it's no longer needed
#[must_use]
pub fn generate_transaction_entry_with_id(
    tx_entry: ScriptTransactionEntry,
    input_bytes: Vec<u8>,
) -> ScriptTransactionEntries {
    let id = generate_id(tx_entry.name.as_str(), input_bytes);
    let transaction = HashMap::from([(id, tx_entry)]);
    ScriptTransactionEntries {
        transactions: transaction,
    }
}

pub fn read_txs_from_state_file(
    state_file_path: &Utf8PathBuf,
) -> Result<Option<ScriptTransactionEntries>> {
    let state_file = load_state_file(state_file_path)?;
    Ok(state_file.transactions)
}

pub fn write_txs_to_state_file(
    state_file_path: &Utf8PathBuf,
    tx_entries: ScriptTransactionEntries,
) -> Result<()> {
    let mut state_file = load_or_create_state_file(state_file_path)
        .with_context(|| anyhow!(format!("Failed to write to state file {state_file_path}")))?;
    state_file.append_transaction_entries(tx_entries);
    fs::write(
        state_file_path,
        serde_json::to_string_pretty(&state_file)
            .expect("Failed to convert ScriptTransactionsSchema to json"),
    )
    .unwrap_or_else(|_| panic!("Failed to write new transactions to state file {state_file_path}"));
    Ok(())
}

fn verify_version(version: u8) -> Result<()> {
    match version {
        STATE_FILE_VERSION => Ok(()),
        _ => Err(anyhow!(format!("Unsupported state file version {version}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::response::structs::Felt;
    use crate::state::state_file::ScriptTransactionOutput::ErrorResponse;
    use camino::Utf8PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_load_or_create_state_file_new_happy() {
        let tempdir = TempDir::new().unwrap();
        let state_file = Utf8PathBuf::from_path_buf(
            tempdir
                .path()
                .join("test_load_or_create_state_file_new_happy.json"),
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

        assert_eq!(
            result.transactions.unwrap().get("123abc789").unwrap().name,
            "declare"
        );
    }

    #[test]
    fn test_load_or_create_state_file_exists_with_txs() {
        let state_file = Utf8PathBuf::from("tests/data/files/state_with_txs.json");
        let result = load_or_create_state_file(&state_file).unwrap();

        let transaction_entry = result
            .transactions
            .as_ref()
            .and_then(|tx| tx.get("789def420"))
            .unwrap();

        match &transaction_entry.output {
            ErrorResponse(error) => assert_eq!(
                error.message,
                "Max fee is smaller than the minimal transaction cost"
            ),
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_load_or_create_state_file_exists_corrupt() {
        let state_file = Utf8PathBuf::from("tests/data/files/state_corrupt_missing_field.json");
        let result = load_or_create_state_file(&state_file).unwrap_err();
        assert_eq!(
            result.to_string(),
            "Failed to parse state file - it may be corrupt"
        );
    }

    #[test]
    #[should_panic(expected = "No such file or directory (os error 2)")]
    fn test_load_state_file_invalid_path() {
        let state_file = Utf8PathBuf::from("bla/bla/crypto.json");
        load_state_file(&state_file).unwrap();
    }

    #[test]
    fn test_version_mismatch() {
        let state_file = Utf8PathBuf::from("tests/data/files/state_wrong_version.json");
        let result = load_or_create_state_file(&state_file).unwrap_err();
        assert_eq!(result.to_string(), "Unsupported state file version 0");
    }

    #[test]
    fn test_write_to_file() {
        let tempdir = TempDir::new().unwrap();
        let state_file_path =
            Utf8PathBuf::from_path_buf(tempdir.path().join("write_state_nofile.json")).unwrap();

        let inputs = vec![123u8, 46u8];
        let transaction = ScriptTransactionEntry {
            name: "declare".to_string(),
            output: ScriptTransactionOutput::DeclareResponse(DeclareResponse {
                class_hash: Felt("0x123".parse().unwrap()),
                transaction_hash: Felt("0x321".parse().unwrap()),
            }),
            status: ScriptTransactionStatus::Success,
            timestamp: 0,
            misc: None,
        };

        let tx_entry = generate_transaction_entry_with_id(transaction.clone(), inputs);
        write_txs_to_state_file(&state_file_path, tx_entry).unwrap();
        let content = fs::read_to_string(state_file_path).unwrap();
        let parsed_content = serde_json::from_str::<ScriptTransactionsSchema>(&content).unwrap();

        assert_eq!(parsed_content.version, 1);
        assert_eq!(
            parsed_content
                .transactions
                .unwrap()
                .transactions
                .iter()
                .next()
                .unwrap()
                .1,
            &transaction
        );
    }

    #[test]
    fn test_write_to_file_append() {
        let tempdir = TempDir::new().unwrap();
        let state_file_path =
            Utf8PathBuf::from_path_buf(tempdir.path().join("write_state_append.json")).unwrap();

        let inputs = vec![123u8, 45u8];
        let transaction1 = ScriptTransactionEntry {
            name: "declare".to_string(),
            output: ScriptTransactionOutput::DeclareResponse(DeclareResponse {
                class_hash: Felt("0x1".parse().unwrap()),
                transaction_hash: Felt("0x2".parse().unwrap()),
            }),
            status: ScriptTransactionStatus::Success,
            timestamp: 0,
            misc: None,
        };

        let tx1_entry = generate_transaction_entry_with_id(transaction1.clone(), inputs);
        write_txs_to_state_file(&state_file_path, tx1_entry).unwrap();

        let inputs = vec![101u8, 22u8];
        let transaction2 = ScriptTransactionEntry {
            name: "invoke".to_string(),
            output: ScriptTransactionOutput::InvokeResponse(InvokeResponse {
                transaction_hash: Felt("0x3".parse().unwrap()),
            }),
            status: ScriptTransactionStatus::Success,
            timestamp: 1,
            misc: None,
        };

        let tx2_entry = generate_transaction_entry_with_id(transaction2.clone(), inputs);
        write_txs_to_state_file(&state_file_path, tx2_entry).unwrap();

        let content = fs::read_to_string(state_file_path).unwrap();
        let parsed_content = serde_json::from_str::<ScriptTransactionsSchema>(&content).unwrap();

        assert_eq!(
            parsed_content
                .transactions
                .clone()
                .unwrap()
                .transactions
                .len(),
            2
        );
        assert_eq!(
            parsed_content
                .transactions
                .clone()
                .unwrap()
                .get("73a528dc325194630de256f187a49c8c3984cdeda6eacc0bad31053cb23715e2")
                .unwrap(),
            &transaction1
        );
        assert_eq!(
            parsed_content
                .transactions
                .clone()
                .unwrap()
                .get("d9b7c5fd12456cbad0a707f3a7800b17f0ad329c9795b4d392a053ef29caa947")
                .unwrap(),
            &transaction2
        );
    }

    #[test]
    fn test_write_to_file_multiple_at_once() {
        let tempdir = TempDir::new().unwrap();
        let state_file_path =
            Utf8PathBuf::from_path_buf(tempdir.path().join("write_state_multiple.json")).unwrap();
        let mut state = ScriptTransactionsSchema {
            version: STATE_FILE_VERSION,
            transactions: None,
        };

        let inputs = vec![123u8, 45u8];
        let transaction1 = ScriptTransactionEntry {
            name: "declare".to_string(),
            output: ScriptTransactionOutput::DeclareResponse(DeclareResponse {
                class_hash: Felt("0x1".parse().unwrap()),
                transaction_hash: Felt("0x2".parse().unwrap()),
            }),
            status: ScriptTransactionStatus::Success,
            timestamp: 2,
            misc: None,
        };
        let tx1_entry = generate_transaction_entry_with_id(transaction1.clone(), inputs);
        state.append_transaction_entries(tx1_entry);

        let inputs = vec![13u8, 15u8];
        let transaction2 = ScriptTransactionEntry {
            name: "invoke".to_string(),
            output: ScriptTransactionOutput::InvokeResponse(InvokeResponse {
                transaction_hash: Felt("0x3".parse().unwrap()),
            }),
            status: ScriptTransactionStatus::Success,
            timestamp: 3,
            misc: None,
        };
        let tx2_entry = generate_transaction_entry_with_id(transaction2.clone(), inputs);
        state.append_transaction_entries(tx2_entry);

        write_txs_to_state_file(&state_file_path, state.transactions.unwrap()).unwrap();

        let content = fs::read_to_string(state_file_path).unwrap();
        let parsed_content = serde_json::from_str::<ScriptTransactionsSchema>(&content).unwrap();

        assert_eq!(
            parsed_content
                .transactions
                .clone()
                .unwrap()
                .transactions
                .len(),
            2
        );
        assert_eq!(
            parsed_content
                .transactions
                .clone()
                .unwrap()
                .get("73a528dc325194630de256f187a49c8c3984cdeda6eacc0bad31053cb23715e2")
                .unwrap(),
            &transaction1
        );
        assert_eq!(
            parsed_content
                .transactions
                .clone()
                .unwrap()
                .get("7b99f490728861b6701d95268e612dd4d0b5bb1f3a9e2dbbe9cc27f3eccda234")
                .unwrap(),
            &transaction2
        );
    }

    #[test]
    fn test_read_and_write_state_file_exists_with_txs() {
        let from_state_file = Utf8PathBuf::from("tests/data/files/state_with_txs.json");
        let tempdir = TempDir::new().unwrap();
        let temp_state_file =
            Utf8PathBuf::from_path_buf(tempdir.path().join("state_with_txs.json")).unwrap();
        fs::copy(from_state_file, &temp_state_file).unwrap();

        let tx_id = "789def420".to_string();

        let result = read_txs_from_state_file(&temp_state_file).expect("Failed to read state file");
        let mut entries = result.unwrap();
        let transaction_entry = entries.transactions.get(&tx_id).unwrap();
        assert_eq!(entries.transactions.len(), 2);
        assert_eq!(transaction_entry.status, ScriptTransactionStatus::Fail);

        let new_transaction = ScriptTransactionEntry {
            name: "deploy".to_string(),
            output: ScriptTransactionOutput::DeployResponse(DeployResponse {
                transaction_hash: Felt("0x3".parse().unwrap()),
                contract_address: Felt("0x333".parse().unwrap()),
            }),
            status: ScriptTransactionStatus::Success,
            timestamp: 1,
            misc: None,
        };
        entries
            .transactions
            .insert(tx_id.clone(), new_transaction)
            .unwrap();
        write_txs_to_state_file(&temp_state_file, entries).unwrap();

        let result = read_txs_from_state_file(&temp_state_file).expect("Failed to read state file");
        let entries = result.unwrap();
        let transaction_entry = entries.transactions.get(&tx_id).unwrap();
        assert_eq!(entries.transactions.len(), 2);
        assert_eq!(transaction_entry.status, ScriptTransactionStatus::Success);
    }
}
