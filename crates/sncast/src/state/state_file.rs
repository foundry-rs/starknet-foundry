#![allow(dead_code)]
use crate::WaitForTransactionError;
use crate::helpers::constants::STATE_FILE_VERSION;
use crate::response::errors::StarknetCommandError;
use crate::response::structs::{DeclareResponse, DeployResponse, InvokeResponse};
use crate::state::hashing::generate_id;
use anyhow::{Context, Result, anyhow};
use camino::Utf8PathBuf;
use conversions::serde::serialize::{BufferWriter, CairoSerialize};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

struct InnerStateManager {
    state_file: Utf8PathBuf,
    executed_transactions_prev_run: ScriptTransactionEntries,
    executed_transactions_current_run: ScriptTransactionEntries,
}

#[derive(Default)]
pub struct StateManager {
    inner: Option<InnerStateManager>,
}

impl StateManager {
    pub fn from(state_file_path: Option<Utf8PathBuf>) -> Result<Self> {
        let res = if let Some(state_file_path) = state_file_path {
            let executed_transactions = load_or_create_state_file(&state_file_path)?
                .transactions
                .unwrap_or_default();

            Self {
                inner: Some(InnerStateManager {
                    state_file: state_file_path,
                    executed_transactions_prev_run: executed_transactions,
                    executed_transactions_current_run: ScriptTransactionEntries::default(),
                }),
            }
        } else {
            Self::default()
        };

        Ok(res)
    }

    #[must_use]
    pub fn get_output_if_success(&self, tx_id: &str) -> Option<ScriptTransactionOutput> {
        if let Some(state) = &self.inner {
            return state
                .executed_transactions_prev_run
                .get_success_output(tx_id);
        }
        None
    }

    pub fn maybe_insert_tx_entry(
        &mut self,
        tx_id: &str,
        selector: &str,
        result: &Result<impl Into<ScriptTransactionOutput> + Clone, StarknetCommandError>,
    ) -> Result<()> {
        if let Some(state) = &mut self.inner {
            state.executed_transactions_current_run.insert(
                tx_id,
                ScriptTransactionEntry::from(selector.to_string(), result),
            );

            write_txs_to_state_file(
                &state.state_file,
                state.executed_transactions_current_run.clone(),
            )?;
        }

        Ok(())
    }
}

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

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Default)]
pub struct ScriptTransactionEntries {
    pub transactions: HashMap<String, ScriptTransactionEntry>,
}

impl ScriptTransactionEntries {
    #[must_use]
    pub fn get(&self, tx_id: &str) -> Option<&ScriptTransactionEntry> {
        self.transactions.get(tx_id)
    }

    pub fn insert(&mut self, tx_id: &str, entry: ScriptTransactionEntry) {
        self.transactions.insert(tx_id.to_string(), entry);
    }

    #[must_use]
    pub fn get_success_output(&self, tx_id: &str) -> Option<ScriptTransactionOutput> {
        if let Some(entry) = self.get(tx_id) {
            if entry.status == ScriptTransactionStatus::Success {
                return Some(entry.output.clone());
            }
        }
        None
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct ScriptTransactionEntry {
    pub name: String,
    pub output: ScriptTransactionOutput,
    pub status: ScriptTransactionStatus,
    pub timestamp: u64,
    pub misc: Option<HashMap<String, Value>>,
}

impl ScriptTransactionEntry {
    pub fn from(
        name: String,
        result: &Result<impl Into<ScriptTransactionOutput> + Clone, StarknetCommandError>,
    ) -> ScriptTransactionEntry {
        let (response, status) = match result {
            Ok(response) => {
                let response: ScriptTransactionOutput = (*response).clone().into();
                (response, ScriptTransactionStatus::Success)
            }
            Err(error) => {
                let transaction_status = match error {
                    StarknetCommandError::WaitForTransactionError(
                        WaitForTransactionError::TransactionError(_),
                    ) => ScriptTransactionStatus::Fail,
                    _ => ScriptTransactionStatus::Error,
                };
                let response = ErrorResponse {
                    message: error.to_string(),
                };
                let response = ScriptTransactionOutput::ErrorResponse(response);
                (response, transaction_status)
            }
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is smaller than Unix epoch")
            .as_secs();

        Self {
            name,
            output: response,
            status,
            timestamp,
            misc: None,
        }
    }
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

impl From<InvokeResponse> for ScriptTransactionOutput {
    fn from(value: InvokeResponse) -> Self {
        Self::InvokeResponse(value)
    }
}

impl From<DeclareResponse> for ScriptTransactionOutput {
    fn from(value: DeclareResponse) -> Self {
        Self::DeclareResponse(value)
    }
}

impl From<DeployResponse> for ScriptTransactionOutput {
    fn from(value: DeployResponse) -> Self {
        Self::DeployResponse(value)
    }
}

impl CairoSerialize for ScriptTransactionOutput {
    fn serialize(&self, output: &mut BufferWriter) {
        match self {
            ScriptTransactionOutput::InvokeResponse(val) => {
                Ok::<_, StarknetCommandError>(val).serialize(output);
            }
            ScriptTransactionOutput::DeclareResponse(val) => {
                Ok::<_, StarknetCommandError>(val).serialize(output);
            }
            ScriptTransactionOutput::DeployResponse(val) => {
                Ok::<_, StarknetCommandError>(val).serialize(output);
            }
            ScriptTransactionOutput::ErrorResponse(_) => {
                panic!("Cannot return ErrorResponse as script function response")
            }
        }
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Clone, Copy, Deserialize, Serialize, Debug, PartialEq)]
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
                .context("Failed to convert ScriptTransactionsSchema to json")?,
        )
        .context("Failed to write initial state to state file")?;
        Ok(default_state)
    }
}

// TODO(#1233): remove must_use attribute when it's no longer needed
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
    .with_context(|| anyhow!("Failed to write new transactions to state file {state_file_path}"))?;
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
    use crate::response::structs::DeclareTransactionResponse;
    use crate::state::state_file::ScriptTransactionOutput::ErrorResponse;
    use camino::Utf8PathBuf;
    use conversions::IntoConv;
    use conversions::string::TryFromHexStr;
    use starknet_types_core::felt::Felt;
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
    fn test_load_or_create_state_file_exists_with_tx_pre_0_34_0() {
        let state_file = Utf8PathBuf::from("tests/data/files/pre_0.34.0_state_with_tx.json");
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
    #[should_panic(expected = "Failed to load state file")]
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
            output: ScriptTransactionOutput::DeclareResponse(DeclareResponse::Success(
                DeclareTransactionResponse {
                    class_hash: Felt::try_from_hex_str("0x123").unwrap().into_(),
                    transaction_hash: Felt::try_from_hex_str("0x321").unwrap().into_(),
                },
            )),
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
            output: ScriptTransactionOutput::DeclareResponse(DeclareResponse::Success(
                DeclareTransactionResponse {
                    class_hash: Felt::try_from_hex_str("0x1").unwrap().into_(),
                    transaction_hash: Felt::try_from_hex_str("0x2").unwrap().into_(),
                },
            )),
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
                transaction_hash: Felt::try_from_hex_str("0x3").unwrap().into_(),
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
            output: ScriptTransactionOutput::DeclareResponse(DeclareResponse::Success(
                DeclareTransactionResponse {
                    class_hash: Felt::try_from_hex_str("0x1").unwrap().into_(),
                    transaction_hash: Felt::try_from_hex_str("0x2").unwrap().into_(),
                },
            )),
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
                transaction_hash: Felt::try_from_hex_str("0x3").unwrap().into_(),
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
        assert_eq!(entries.transactions.len(), 3);
        assert_eq!(transaction_entry.status, ScriptTransactionStatus::Fail);

        let new_transaction = ScriptTransactionEntry {
            name: "deploy".to_string(),
            output: ScriptTransactionOutput::DeployResponse(DeployResponse {
                transaction_hash: Felt::try_from_hex_str("0x3").unwrap().into_(),
                contract_address: Felt::try_from_hex_str("0x333").unwrap().into_(),
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
        assert_eq!(entries.transactions.len(), 3);
        assert_eq!(transaction_entry.status, ScriptTransactionStatus::Success);
    }
}
