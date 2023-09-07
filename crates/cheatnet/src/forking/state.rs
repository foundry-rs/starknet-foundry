use blockifier::execution::contract_class::{
    ContractClass as ContractClassBlockifier, ContractClassV1,
};
use blockifier::state::errors::StateError::{StateReadError, UndeclaredClassHash};
use blockifier::state::state_api::{StateReader, StateResult};
use cairo_lang_starknet::abi::Contract;
use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_lang_starknet::contract_class::{ContractClass, ContractEntryPoints};
use cairo_lang_utils::bigint::BigUintAsHex;
use conversions::StarknetConversions;
use num_bigint::BigUint;
use starknet::core::types::{BlockId, ContractClass as ContractClassStarknet};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};
use starknet_api::core::{ClassHash, CompiledClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use tokio::runtime::Runtime;
use url::Url;

#[derive(Debug)]
pub struct ForkStateReader {
    client: JsonRpcClient<HttpTransport>,
    block_id: BlockId,
    runtime: Runtime,
}

impl ForkStateReader {
    #[must_use]
    pub fn new(url: &str, block_id: BlockId) -> Self {
        ForkStateReader {
            client: JsonRpcClient::new(HttpTransport::new(Url::parse(url).unwrap())),
            block_id,
            runtime: Runtime::new().expect("Could not instantiate Runtime"),
        }
    }
}

impl StateReader for ForkStateReader {
    fn get_storage_at(
        &mut self,
        contract_address: ContractAddress,
        key: StorageKey,
    ) -> StateResult<StarkFelt> {
        match self.runtime.block_on(self.client.get_storage_at(
            contract_address.to_field_element(),
            key.0.key().to_field_element(),
            self.block_id,
        )) {
            Ok(value) => Ok(StarkFelt::from(value)),
            Err(error) => Err(StateReadError(error.to_string())),
        }
    }

    fn get_nonce_at(&mut self, contract_address: ContractAddress) -> StateResult<Nonce> {
        match self.runtime.block_on(
            self.client
                .get_nonce(self.block_id, contract_address.to_field_element()),
        ) {
            Ok(nonce) => Ok(Nonce(StarkFelt::from(nonce))),
            Err(error) => Err(StateReadError(error.to_string())),
        }
    }

    fn get_class_hash_at(&mut self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        match self.runtime.block_on(
            self.client
                .get_class_hash_at(self.block_id, contract_address.to_field_element()),
        ) {
            Ok(class_hash) => Ok(class_hash.to_class_hash()),
            Err(error) => Err(StateReadError(error.to_string())),
        }
    }

    fn get_compiled_contract_class(
        &mut self,
        class_hash: &ClassHash,
    ) -> StateResult<ContractClassBlockifier> {
        let contract_class = self.runtime.block_on(
            self.client
                .get_class(self.block_id, class_hash.to_field_element()),
        );

        if contract_class.is_err() {
            return Err(UndeclaredClassHash(*class_hash));
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

    fn get_compiled_class_hash(
        &mut self,
        _class_hash: ClassHash,
    ) -> StateResult<CompiledClassHash> {
        todo!()
    }
}
