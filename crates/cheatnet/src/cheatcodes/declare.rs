use std::collections::HashMap;

use crate::constants::{
    build_block_context, build_declare_transaction, TEST_ACCOUNT_CONTRACT_ADDRESS,
};
use crate::state::CustomStateReader;
use crate::{
    cheatcodes::{CheatcodeError, ContractArtifacts, EnhancedHintError},
    CheatnetState,
};
use anyhow::{anyhow, Context, Result};
use blockifier::execution::contract_class::{
    ContractClass as BlockifierContractClass, ContractClassV1,
};
use blockifier::state::cached_state::CachedState;
use blockifier::state::state_api::StateReader;
use blockifier::transaction::account_transaction::AccountTransaction;
use blockifier::transaction::transactions::{DeclareTransaction, ExecutableTransaction};
use cairo_felt::Felt252;
use serde_json;
use starknet_api::core::{ClassHash, ContractAddress, PatriciaKey};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::patricia_key;
use starknet_api::transaction::TransactionHash;

use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_lang_starknet::contract_class::ContractClass;
use starknet::core::types::contract::CompiledClass;

impl CheatnetState {
    pub fn declare(
        &mut self,
        contract_name: &Felt252,
        contracts: &HashMap<String, ContractArtifacts>,
    ) -> Result<ClassHash, CheatcodeError> {
        let blockifier_state: &mut CachedState<CustomStateReader> = &mut self.blockifier_state;

        let contract_name_as_short_str = as_cairo_short_string(contract_name)
            .context("Converting contract name to short string failed")
            .map_err::<EnhancedHintError, _>(From::from)?;
        let contract_artifact = contracts.get(&contract_name_as_short_str).with_context(|| {
            format!("Failed to get contract artifact for name = {contract_name_as_short_str}. Make sure starknet target is correctly defined in Scarb.toml file.")
        }).map_err::<EnhancedHintError, _>(From::from)?;

        let sierra_contract_class: ContractClass = serde_json::from_str(&contract_artifact.sierra)
            .unwrap_or_else(|_| {
                panic!("Failed to parse json from artifact = {contract_artifact:?}")
            });

        let casm_contract_class =
            CasmContractClass::from_contract_class(sierra_contract_class, true)
                .expect("Sierra to casm failed");
        let casm_serialized = serde_json::to_string_pretty(&casm_contract_class)
            .expect("Failed to serialize contract to casm");

        let contract_class = ContractClassV1::try_from_json_string(&casm_serialized)
            .expect("Failed to read contract class from json");
        let contract_class = BlockifierContractClass::V1(contract_class);

        let class_hash =
            get_class_hash(casm_serialized.as_str()).expect("Failed to get class hash");

        let nonce = blockifier_state
            .get_nonce_at(ContractAddress(patricia_key!(
                TEST_ACCOUNT_CONTRACT_ADDRESS
            )))
            .expect("Failed to get nonce");

        let declare_tx = build_declare_transaction(
            nonce,
            class_hash,
            ContractAddress(patricia_key!(TEST_ACCOUNT_CONTRACT_ADDRESS)),
        );
        let tx = DeclareTransaction::new(
            starknet_api::transaction::DeclareTransaction::V2(declare_tx),
            // TODO(#358)
            TransactionHash::default(),
            contract_class,
        )
        .unwrap_or_else(|err| panic!("Unable to build transaction {err:?}"));

        let account_tx = AccountTransaction::Declare(tx);
        let block_context = build_block_context();
        match account_tx.execute(blockifier_state, &block_context, true, true) {
            Ok(_) => (),
            Err(e) => {
                return Err(EnhancedHintError::Anyhow(anyhow!(format!(
                    "Failed to execute declare transaction:\n    {e}"
                ))))
                .map_err(From::from)
            }
        };

        Ok(class_hash)
    }
}

fn get_class_hash(casm_contract: &str) -> Result<ClassHash> {
    let compiled_class = serde_json::from_str::<CompiledClass>(casm_contract)?;
    let class_hash = compiled_class.class_hash()?;
    let class_hash = StarkFelt::new(class_hash.to_bytes_be())?;
    Ok(ClassHash(class_hash))
}
