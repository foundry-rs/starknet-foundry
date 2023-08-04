use std::collections::HashMap;

use crate::constants::{
    build_block_context, build_declare_transaction, TEST_ACCOUNT_CONTRACT_ADDRESS,
};
use crate::state::DictStateReader;
use crate::{
    cheatcodes::{ContractArtifacts, EnhancedHintError},
    CheatedState,
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
use num_traits::Num;
use serde_json;
use starknet_api::core::{ClassHash, ContractAddress, PatriciaKey};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::patricia_key;
use starknet_api::transaction::TransactionHash;

use cairo_lang_runner::casm_run::MemBuffer;
use cairo_lang_runner::short_string::as_cairo_short_string;
use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_lang_starknet::contract_class::ContractClass;
use starknet::core::types::contract::CompiledClass;

impl CheatedState {
    pub fn declare(
        &self,
        buffer: &mut MemBuffer,
        blockifier_state: &mut CachedState<DictStateReader>,
        inputs: &[Felt252],
        contracts: &HashMap<String, ContractArtifacts>,
    ) -> Result<(), EnhancedHintError> {
        let contract_value = inputs[0].clone();

        let contract_value_as_short_str = as_cairo_short_string(&contract_value)
            .context("Converting contract name to short string failed")?;
        let contract_artifact = contracts.get(&contract_value_as_short_str).ok_or_else(|| {
            anyhow!("Failed to get contract artifact for name = {contract_value_as_short_str}. Make sure starknet target is correctly defined in Scarb.toml file.")
        })?;
        let sierra_contract_class: ContractClass = serde_json::from_str(&contract_artifact.sierra)
            .with_context(|| format!("File to parse json from artifact = {contract_artifact:?}"))?;

        let casm_contract_class =
            CasmContractClass::from_contract_class(sierra_contract_class, true)
                .context("Sierra to casm failed")?;
        let casm_serialized = serde_json::to_string_pretty(&casm_contract_class)
            .context("Failed to serialize contract to casm")?;

        let contract_class = ContractClassV1::try_from_json_string(&casm_serialized)
            .context("Failed to read contract class from json")?;
        let contract_class = BlockifierContractClass::V1(contract_class);

        let class_hash = get_class_hash(casm_serialized.as_str())?;

        let nonce = blockifier_state
            .get_nonce_at(ContractAddress(patricia_key!(
                TEST_ACCOUNT_CONTRACT_ADDRESS
            )))
            .context("Failed to get nonce")?;

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
                return Err(
                    anyhow!(format!("Failed to execute declare transaction:\n    {e}")).into(),
                )
            }
        };

        // result_segment.
        let felt_class_hash = felt252_from_hex_string(&class_hash.to_string()).unwrap();

        buffer
            .write(Felt252::from(0))
            .expect("Failed to insert error code");
        buffer
            .write(felt_class_hash)
            .expect("Failed to insert declared contract class hash");

        Ok(())
    }
}

fn get_class_hash(casm_contract: &str) -> Result<ClassHash> {
    let compiled_class = serde_json::from_str::<CompiledClass>(casm_contract)?;
    let class_hash = compiled_class.class_hash()?;
    let class_hash = StarkFelt::new(class_hash.to_bytes_be())?;
    Ok(ClassHash(class_hash))
}

fn felt252_from_hex_string(value: &str) -> Result<Felt252> {
    let stripped_value = value.replace("0x", "");
    Felt252::from_str_radix(&stripped_value, 16)
        .map_err(|_| anyhow!("Failed to convert value = {value} to Felt252"))
}

#[cfg(test)]
mod test {
    use assert_fs::fixture::{FileWriteStr, PathChild, PathCopy};
    use assert_fs::TempDir;
    use camino::Utf8PathBuf;
    use indoc::formatdoc;
    use starknet_api::stark_felt;
    use std::process::Command;
    use std::str::FromStr;

    use super::*;

    #[test]
    fn felt_2525_from_prefixed_hex() {
        assert_eq!(
            felt252_from_hex_string("0x1234").unwrap(),
            Felt252::from(0x1234)
        );
    }

    #[test]
    fn felt_2525_from_non_prefixed_hex() {
        assert_eq!(
            felt252_from_hex_string("1234").unwrap(),
            Felt252::from(0x1234)
        );
    }

    #[test]
    fn felt_252_err_on_failed_conversion() {
        let result = felt252_from_hex_string("yyyy");
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "Failed to convert value = yyyy to Felt252");
    }

    #[test]
    fn class_hash_correct() {
        let temp = TempDir::new().unwrap();
        // TODO(#305) change to cheatnet data path
        temp.copy_from(
            "../forge/tests/data/simple_package",
            &["**/*.cairo", "**/*.toml"],
        )
        .unwrap();

        let cheatcodes_path = Utf8PathBuf::from_str("../../cheatcodes")
            .unwrap()
            .canonicalize_utf8()
            .unwrap();

        let manifest_path = temp.child("Scarb.toml");
        manifest_path
            .write_str(&formatdoc!(
                r#"
                [package]
                name = "simple_package"
                version = "0.1.0"
        
                [[target.starknet-contract]]
                sierra = true
                casm = true
        
                [dependencies]
                starknet = "2.1.0-rc2"
                cheatcodes = {{ path = "{}" }}
                "#,
                cheatcodes_path
            ))
            .unwrap();

        Command::new("scarb")
            .current_dir(&temp)
            .arg("build")
            .output()
            .unwrap();

        let temp_dir_path = temp.path();

        // expected_class_hash computed with
        // https://github.com/software-mansion/starknet.py/blob/cea191679cbdd2726ca7989f3a7662dee6ea43ca/starknet_py/tests/e2e/docs/guide/test_cairo1_contract.py#L29-L36
        let cases = [
            (
                // TODO(#369) verify calculation of this
                "0x00b0e07d0ab5d68a22072cd5f35f39335d0dcbf1a28fb92820bd5d547c497f33",
                "target/dev/simple_package_ERC20.casm.json",
            ),
            (
                // TODO(#369) verify calculation of this
                "0x02ff90d068517ba09883a50339d55cc8a678a60e1526032ee0a899ed219f44e7",
                "target/dev/simple_package_HelloStarknet.casm.json",
            ),
        ];

        for (expected_class_hash, casm_contract_path) in cases {
            let casm_contract_path = temp_dir_path.join(casm_contract_path);
            let casm_contract_path = casm_contract_path.as_path();

            let casm_contract_definition = std::fs::read_to_string(casm_contract_path)
                .unwrap_or_else(|_| panic!("Failed to read file: {casm_contract_path:?}"));
            let actual_class_hash = get_class_hash(casm_contract_definition.as_str()).unwrap();
            assert_eq!(
                actual_class_hash,
                ClassHash(stark_felt!(expected_class_hash))
            );
        }
    }
}
