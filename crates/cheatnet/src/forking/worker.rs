use crate::conversions::{class_hash_to_felt, contract_address_to_felt};
use blockifier::execution::contract_class::{
    ContractClass as ContractClassBlockifier, ContractClassV1,
};
use blockifier::execution::execution_utils::stark_felt_to_felt;
use blockifier::state::errors::StateError::StateReadError;
use blockifier::state::state_api::StateResult;
use cairo_lang_starknet::abi::Contract;
use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_lang_starknet::contract_class::{ContractClass, ContractEntryPoints};
use cairo_lang_utils::bigint::BigUintAsHex;
use num_bigint::BigUint;
use starknet::core::types::BlockTag::Latest;
use starknet::core::types::{BlockId, ContractClass as ContractClassStarknet, FieldElement};
use starknet::providers::jsonrpc::{HttpTransport};
use starknet::providers::{JsonRpcClient, Provider};
use starknet_api::core::{ClassHash, ContractAddress, Nonce};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::state::StorageKey;
use tokio::runtime::Runtime;
use url::Url;

#[derive(Debug)]
pub struct Worker {
    client: JsonRpcClient<HttpTransport>,
}

impl Worker {
    #[must_use]
    pub fn new(url: &str) -> Self {
        Worker {
            client: JsonRpcClient::new(HttpTransport::new(Url::parse(url).unwrap())),
        }
    }

    pub fn get_nonce(&self, contract_address: ContractAddress) -> StateResult<Nonce> {
        let rt = Runtime::new().expect("Could not instantiate Runtime");
        match rt.block_on(
            self.client.get_nonce(
                BlockId::Tag(Latest),
                FieldElement::from_bytes_be(
                    &contract_address_to_felt(contract_address).to_be_bytes(),
                )
                .unwrap(),
            ),
        ) {
            Ok(nonce) => Ok(Nonce(StarkFelt::from(nonce))),
            Err(error) => Err(StateReadError(error.to_string())),
        }
    }

    pub fn get_class_hash_at(&self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        let rt = Runtime::new().expect("Could not instantiate Runtime");
        match rt.block_on(
            self.client.get_class_hash_at(
                BlockId::Tag(Latest),
                FieldElement::from_bytes_be(
                    &contract_address_to_felt(contract_address).to_be_bytes(),
                )
                .unwrap(),
            ),
        ) {
            Ok(class_hash) => Ok(ClassHash(StarkHash::from(class_hash))),
            Err(error) => Err(StateReadError(error.to_string())),
        }
    }

    pub fn get_storage_at(
        &self,
        contract_address: ContractAddress,
        key: StorageKey,
    ) -> StateResult<StarkFelt> {
        let rt = Runtime::new().expect("Could not instantiate Runtime");
        match rt.block_on(
            self.client.get_storage_at(
                FieldElement::from_bytes_be(
                    &contract_address_to_felt(contract_address).to_be_bytes(),
                )
                .unwrap(),
                FieldElement::from_bytes_be(&stark_felt_to_felt(*key.0.key()).to_be_bytes())
                    .unwrap(),
                BlockId::Tag(Latest),
            ),
        ) {
            Ok(value) => Ok(StarkFelt::from(value)),
            Err(error) => Err(StateReadError(error.to_string())),
        }
    }

    pub fn get_compiled_contract_class(
        &mut self,
        class_hash: &ClassHash,
    ) -> StateResult<ContractClassBlockifier> {
        let rt = Runtime::new().expect("Could not instantiate Runtime");
        let contract_class = rt.block_on(self.client.get_class(
            BlockId::Tag(Latest),
            FieldElement::from_bytes_be(&class_hash_to_felt(*class_hash).to_be_bytes()).unwrap(),
        ));

        if let Err(error) = contract_class {
            return Err(StateReadError(error.to_string()));
        }

        match contract_class.unwrap() {
            ContractClassStarknet::Sierra(flattened_class) => {
                let converted_sierra_program: Vec<BigUintAsHex> = flattened_class
                    .sierra_program
                    .iter()
                    .map(|field_element| BigUintAsHex {
                        value: BigUint::from_bytes_be(&field_element.to_bytes_be()),
                    })
                    .collect();
                let converted_entry_points: ContractEntryPoints = serde_json::from_str(
                    &serde_json::to_string(&flattened_class.entry_points_by_type).unwrap(),
                )
                .unwrap();
                let converted_abi: Contract = serde_json::from_str(&flattened_class.abi).unwrap();

                let sierra_contract_class: ContractClass = ContractClass {
                    sierra_program: converted_sierra_program,
                    sierra_program_debug_info: None,
                    contract_class_version: flattened_class.contract_class_version,
                    entry_points_by_type: converted_entry_points,
                    abi: Some(converted_abi),
                };
                let casm_contract_class: CasmContractClass =
                    CasmContractClass::from_contract_class(sierra_contract_class, false).unwrap();

                Ok(ContractClassBlockifier::V1(
                    ContractClassV1::try_from(casm_contract_class).unwrap(),
                ))
            }
            ContractClassStarknet::Legacy(_) => Err(StateReadError(
                "Cairo 0 contracts are not supported".to_string(),
            )),
        }
    }
}
