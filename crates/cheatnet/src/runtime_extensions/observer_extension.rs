use std::{collections::HashMap, marker::PhantomData};

use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;

use runtime::{utils::BufferReader, CheatcodeHandlingResult, EnhancedHintError, ExtendedRuntime, ExtensionLogic, StarknetRuntime};
use scarb_api::StarknetContractArtifacts;
use starknet_api::core::ClassHash;

use super::forge_runtime_extension::cheatcodes::declare::get_class_hash;


pub struct ObservedInformation {
    contract_name_to_class_hash: HashMap<String, ClassHash>
}

pub struct ObserverExtension<'a> {
    pub contracts: &'a HashMap<String, StarknetContractArtifacts>,
    pub observed_information: ObservedInformation,
}

pub type ObserverRuntime<'a> = ExtendedRuntime<ObserverExtension<'a>>;

impl<'a> ExtensionLogic for ObserverExtension<'a> {
    type Runtime = StarknetRuntime<'a>;

    fn listen_cheatcode(
            &mut self,
            selector: &str,
            _vm: &mut cairo_vm::vm::vm_core::VirtualMachine,
            extended_runtime: &mut Self::Runtime,
        ) {
            let inputs = vec![]; // TODO modify args
            let mut reader = BufferReader::new(&inputs);
            match selector {
                "declare" => {
                    skip_if_error(
                        || {
                            let contract_name = reader.read_felt();
                            let contracts = self.contracts;
        
                            let contract_name_as_short_str = as_cairo_short_string(&contract_name)?;
                            let contract_artifact = contracts.get(&contract_name_as_short_str)?;
                            let class_hash = get_class_hash(contract_artifact.sierra.as_str()).expect("Failed to get class hash");
                            self.observed_information.contract_name_to_class_hash.insert(contract_name_as_short_str, class_hash);
                            Ok(())
                        }
                    )
     
                }
            }
    }
}

fn skip_if_error<T, E>(block: impl Fn() -> Result<T, E>) {
    block();
}
