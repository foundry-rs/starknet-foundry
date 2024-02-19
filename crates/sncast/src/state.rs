use crate::helpers::constants::STATE_FILE_VERSION;
use crate::response::structs::{DeclareResponse, DeployResponse, InvokeResponse};
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

pub fn load_or_create_state_file(path: &Utf8PathBuf) -> Result<ScriptTransactionsSchema> {
    if path.exists() {
        let content = fs::read_to_string(path).expect("Failed to read state file");
        match serde_json::from_str::<ScriptTransactionsSchema>(&content) {
            Ok(state_file) => {
                verify_version(state_file.version)?;
                Ok(state_file)
            }
            Err(_) => Err(anyhow!("Failed to parse state file - it may be corrupt")),
        }
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
) -> ScriptTransactionEntries {
    // todo (1545): Implement hashing function for unique ids
    let id = format!("{}-{}", tx_entry.name, tx_entry.timestamp);
    let transaction = HashMap::from([(id, tx_entry)]);
    ScriptTransactionEntries {
        transactions: transaction,
    }
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
    use crate::response::structs::Hex;
    use crate::state::ScriptTransactionOutput::ErrorResponse;
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
            result
                .transactions
                .unwrap()
                .get("123-abc-789")
                .unwrap()
                .name,
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
            .and_then(|tx| tx.get("789-def-420"))
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

        let transaction = ScriptTransactionEntry {
            name: "declare".to_string(),
            output: ScriptTransactionOutput::DeclareResponse(DeclareResponse {
                class_hash: Hex("0x123".parse().unwrap()),
                transaction_hash: Hex("0x321".parse().unwrap()),
            }),
            status: ScriptTransactionStatus::Success,
            timestamp: 0,
            misc: None,
        };

        let tx_entry = generate_transaction_entry_with_id(transaction.clone());
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

        let transaction1 = ScriptTransactionEntry {
            name: "declare".to_string(),
            output: ScriptTransactionOutput::DeclareResponse(DeclareResponse {
                class_hash: Hex("0x1".parse().unwrap()),
                transaction_hash: Hex("0x2".parse().unwrap()),
            }),
            status: ScriptTransactionStatus::Success,
            timestamp: 0,
            misc: None,
        };

        let tx1_entry = generate_transaction_entry_with_id(transaction1.clone());
        write_txs_to_state_file(&state_file_path, tx1_entry).unwrap();

        let transaction2 = ScriptTransactionEntry {
            name: "invoke".to_string(),
            output: ScriptTransactionOutput::InvokeResponse(InvokeResponse {
                transaction_hash: Hex("0x3".parse().unwrap()),
            }),
            status: ScriptTransactionStatus::Success,
            timestamp: 1,
            misc: None,
        };

        let tx2_entry = generate_transaction_entry_with_id(transaction2.clone());
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
                .get("declare-0")
                .unwrap(),
            &transaction1
        );
        assert_eq!(
            parsed_content
                .transactions
                .clone()
                .unwrap()
                .get("invoke-1")
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

        let transaction1 = ScriptTransactionEntry {
            name: "declare".to_string(),
            output: ScriptTransactionOutput::DeclareResponse(DeclareResponse {
                class_hash: Hex("0x1".parse().unwrap()),
                transaction_hash: Hex("0x2".parse().unwrap()),
            }),
            status: ScriptTransactionStatus::Success,
            timestamp: 2,
            misc: None,
        };
        let tx1_entry = generate_transaction_entry_with_id(transaction1.clone());
        state.append_transaction_entries(tx1_entry);

        let transaction2 = ScriptTransactionEntry {
            name: "invoke".to_string(),
            output: ScriptTransactionOutput::InvokeResponse(InvokeResponse {
                transaction_hash: Hex("0x3".parse().unwrap()),
            }),
            status: ScriptTransactionStatus::Success,
            timestamp: 3,
            misc: None,
        };
        let tx2_entry = generate_transaction_entry_with_id(transaction2.clone());
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
                .get("declare-2")
                .unwrap(),
            &transaction1
        );
        assert_eq!(
            parsed_content
                .transactions
                .clone()
                .unwrap()
                .get("invoke-3")
                .unwrap(),
            &transaction2
        );
    }
}
