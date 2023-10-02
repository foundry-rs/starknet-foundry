use std::collections::HashMap;
use std::ops::DerefMut;

use crate::cheatcodes::{CheatcodeError, ContractArtifacts, EnhancedHintError};
use anyhow::{Context, Result};
use blockifier::execution::contract_class::{
    ContractClass as BlockifierContractClass, ContractClassV1,
};
use blockifier::state::state_api::State;
use cairo_felt::Felt252;
use serde_json;
use starknet_api::core::ClassHash;
use starknet_api::hash::StarkFelt;

use crate::state::BlockifierState;
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_starknet::contract_class::ContractClass;
use serde_json::Value;
use starknet::core::types::FlattenedSierraClass;

impl BlockifierState<'_> {
    pub fn declare(
        &mut self,
        contract_name: &Felt252,
        contracts: &HashMap<String, ContractArtifacts>,
    ) -> Result<ClassHash, CheatcodeError> {
        let blockifier_state: &mut dyn State = self.blockifier_state.deref_mut();

        let contract_name_as_short_str = as_cairo_short_string(contract_name)
            .context("Converting contract name to short string failed")
            .map_err::<EnhancedHintError, _>(From::from)?;
        let contract_artifact = contracts.get(&contract_name_as_short_str).with_context(|| {
            format!("Failed to get contract artifact for name = {contract_name_as_short_str}. Make sure starknet target is correctly defined in Scarb.toml file.")
        }).map_err::<EnhancedHintError, _>(From::from)?;

        let contract_class = ContractClassV1::try_from_json_string(&contract_artifact.casm)
            .expect("Failed to read contract class from json");
        let contract_class = BlockifierContractClass::V1(contract_class);

        let class_hash =
            get_class_hash(contract_artifact.sierra.as_str()).expect("Failed to get class hash");

        // let nonce = blockifier_state
        //     .get_nonce_at(ContractAddress(patricia_key!(
        //         TEST_ACCOUNT_CONTRACT_ADDRESS
        //     )))
        //     .expect("Failed to get nonce");

        // let declare_tx = build_declare_transaction(
        //     nonce,
        //     class_hash,
        //     ContractAddress(patricia_key!(TEST_ACCOUNT_CONTRACT_ADDRESS)),
        // );
        // let tx = DeclareTransaction::new(
        //     starknet_api::transaction::DeclareTransaction::V2(declare_tx),
        //     // TODO(#358)
        //     TransactionHash::default(),
        //     contract_class.clone(),
        // )
        // .unwrap_or_else(|err| panic!("Unable to build transaction {err:?}"));
        // TODO make it correct with compiled class hash
        blockifier_state
            .set_contract_class(&class_hash, contract_class)
            .map_err(EnhancedHintError::from)?;

        // match blockifier_state.get_compiled_contract_class(&class_hash) {
        //     Err(StateError::UndeclaredClassHash(_)) => {
        //         // Class is undeclared; declare it.

        //         Ok(class_hash)
        //         // blockifier_state.set_compiled_class_hash(class_hash, *compiled_class_hash)?;
        //     }
        //     Err(error) => Err(error).map_err(TransactionExecutionError::from),
        //     Ok(_) => {
        //         // Class is already declared, cannot redeclare
        //         // (i.e., make sure the leaf is uninitialized).
        //         Err(TransactionExecutionError::DeclareTransactionError { class_hash })
        //     }
        // }

        Ok(class_hash)
    }
}

fn get_class_hash(sierra_contract: &str) -> Result<ClassHash> {
    let sierra_class: ContractClass = serde_json::from_str(sierra_contract)?;
    let abi_flattened = sierra_class.abi.unwrap().json();
    let mut sierra_contract_map: HashMap<String, Value> = serde_json::from_str(sierra_contract)?;
    sierra_contract_map.insert("abi".to_string(), Value::String(abi_flattened));
    let sierra_contract = serde_json::to_string_pretty(&sierra_contract_map)?;

    let sierra_class: FlattenedSierraClass = serde_json::from_str(&sierra_contract)?;
    let class_hash = sierra_class.class_hash();
    let class_hash = StarkFelt::new(class_hash.to_bytes_be())?;
    Ok(ClassHash(class_hash))
}
