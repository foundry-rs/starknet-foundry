use std::collections::HashMap;

use anyhow::{Context, Error};
use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;

use runtime::{utils::BufferReader, ExtendedRuntime, ExtensionLogic, StarknetRuntime};
use scarb_api::StarknetContractArtifacts;
use starknet_api::core::ClassHash;

use super::forge_runtime_extension::cheatcodes::declare::get_class_hash;

#[derive(Default)]
pub struct ObserverState {
    pub contracts: HashMap<String, StarknetContractArtifacts>,
    pub contract_name_to_class_hash: HashMap<String, ClassHash>,
}

pub struct ObserverExtension<'a> {
    pub observer_state: &'a mut ObserverState,
}

impl<'a> ObserverExtension<'a> {
    pub fn from(observer_state: &'a mut ObserverState) -> Self {
        ObserverExtension { observer_state }
    }
}

pub type ObserverRuntime<'a> = ExtendedRuntime<ObserverExtension<'a>>;

impl<'a> ExtensionLogic for ObserverExtension<'a> {
    type Runtime = StarknetRuntime<'a>;

    fn handle_cheatcode_signal(
        &mut self,
        selector: &str,
        inputs: Vec<Felt252>,
        _extended_runtime: &mut Self::Runtime,
    ) {
        if selector == "declare" {
            skip_if_error(|| {
                let mut reader = BufferReader::new(&inputs);
                let contract_name = reader.read_felt();
                let contracts = &mut self.observer_state.contracts;

                let contract_name_as_short_str = as_cairo_short_string(&contract_name)
                    .context("Converting contract name to short string failed")?;
                let contract_artifact = contracts.get(&contract_name_as_short_str).with_context(|| {
                    format!("Failed to get contract artifact for name = {contract_name_as_short_str}. Make sure starknet target is correctly defined in Scarb.toml file.")})?;
                let class_hash = get_class_hash(contract_artifact.sierra.as_str())
                    .expect("Failed to get class hash");
                self.observer_state
                    .contract_name_to_class_hash
                    .insert(contract_name_as_short_str, class_hash);
                Ok(())
            });
        }
    }
}

fn skip_if_error(mut block: impl FnMut() -> Result<(), Error>) {
    let _ = block();
}
