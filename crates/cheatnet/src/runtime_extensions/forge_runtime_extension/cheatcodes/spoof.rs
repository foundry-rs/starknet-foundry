use super::cheat_execution_info::{CheatArguments, ExecutionInfoMockOperations, Operation, TxInfoMockOperations};
use crate::state::CheatSpan;
use crate::CheatnetState;
use cairo_felt::Felt252;
use conversions::felt252::SerializeAsFelt252Vec;
use runtime::FromReader;
use starknet_api::core::ContractAddress;

#[derive(FromReader, Clone, Default, Debug, Eq, PartialEq)]
pub struct ResourceBounds {
    pub resource: Felt252,
    pub max_amount: u64,
    pub max_price_per_unit: u128,
}

impl SerializeAsFelt252Vec for ResourceBounds {
    fn serialize_into_felt252_vec(self, output: &mut Vec<Felt252>) {
        output.push(self.resource);
        output.push(self.max_amount.into());
        output.push(self.max_price_per_unit.into());
    }
}

#[derive(FromReader, Clone, Default, Debug)]
pub struct TxInfoMock {
    pub version: Option<Felt252>,
    pub account_contract_address: Option<Felt252>,
    pub max_fee: Option<Felt252>,
    pub signature: Option<Vec<Felt252>>,
    pub transaction_hash: Option<Felt252>,
    pub chain_id: Option<Felt252>,
    pub nonce: Option<Felt252>,
    pub resource_bounds: Option<Vec<ResourceBounds>>,
    pub tip: Option<Felt252>,
    pub paymaster_data: Option<Vec<Felt252>>,
    pub nonce_data_availability_mode: Option<Felt252>,
    pub fee_data_availability_mode: Option<Felt252>,
    pub account_deployment_data: Option<Vec<Felt252>>,
}

impl CheatnetState {
    pub fn spoof(
        &mut self,
        contract_address: ContractAddress,
        tx_info_mock: TxInfoMock,
        span: CheatSpan,
    ) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            #[rustfmt::skip]
            tx_info: TxInfoMockOperations {
                version                      :tx_info_mock.version                     .map_or(Operation::Retain, |value| Operation::Start(CheatArguments { value, span, target:contract_address })),             
                account_contract_address     :tx_info_mock.account_contract_address    .map_or(Operation::Retain, |value| Operation::Start(CheatArguments { value, span, target:contract_address })),                            
                max_fee                      :tx_info_mock.max_fee                     .map_or(Operation::Retain, |value| Operation::Start(CheatArguments { value, span, target:contract_address })),           
                signature                    :tx_info_mock.signature                   .map_or(Operation::Retain, |value| Operation::Start(CheatArguments { value, span, target:contract_address })),             
                transaction_hash             :tx_info_mock.transaction_hash            .map_or(Operation::Retain, |value| Operation::Start(CheatArguments { value, span, target:contract_address })),                    
                chain_id                     :tx_info_mock.chain_id                    .map_or(Operation::Retain, |value| Operation::Start(CheatArguments { value, span, target:contract_address })),            
                nonce                        :tx_info_mock.nonce                       .map_or(Operation::Retain, |value| Operation::Start(CheatArguments { value, span, target:contract_address })),         
                resource_bounds              :tx_info_mock.resource_bounds             .map_or(Operation::Retain, |value| Operation::Start(CheatArguments { value, span, target:contract_address })),                   
                tip                          :tx_info_mock.tip                         .map_or(Operation::Retain, |value| Operation::Start(CheatArguments { value, span, target:contract_address })),       
                paymaster_data               :tx_info_mock.paymaster_data              .map_or(Operation::Retain, |value| Operation::Start(CheatArguments { value, span, target:contract_address })),                  
                nonce_data_availability_mode :tx_info_mock.nonce_data_availability_mode.map_or(Operation::Retain, |value| Operation::Start(CheatArguments { value, span, target:contract_address })),                                
                fee_data_availability_mode   :tx_info_mock.fee_data_availability_mode  .map_or(Operation::Retain, |value| Operation::Start(CheatArguments { value, span, target:contract_address })),                              
                account_deployment_data      :tx_info_mock.account_deployment_data     .map_or(Operation::Retain, |value| Operation::Start(CheatArguments { value, span, target:contract_address })),                           
            },
            ..Default::default()
        });
    }

    pub fn spoof_global(
        &mut self,
        tx_info_mock: TxInfoMock,
    ) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            #[rustfmt::skip]
            tx_info: TxInfoMockOperations {
                version                      :tx_info_mock.version                     .map_or(Operation::Retain, Operation::StartGlobal),
                account_contract_address     :tx_info_mock.account_contract_address    .map_or(Operation::Retain, Operation::StartGlobal),
                max_fee                      :tx_info_mock.max_fee                     .map_or(Operation::Retain, Operation::StartGlobal),
                signature                    :tx_info_mock.signature                   .map_or(Operation::Retain, Operation::StartGlobal),
                transaction_hash             :tx_info_mock.transaction_hash            .map_or(Operation::Retain, Operation::StartGlobal),
                chain_id                     :tx_info_mock.chain_id                    .map_or(Operation::Retain, Operation::StartGlobal),
                nonce                        :tx_info_mock.nonce                       .map_or(Operation::Retain, Operation::StartGlobal),
                resource_bounds              :tx_info_mock.resource_bounds             .map_or(Operation::Retain, Operation::StartGlobal),
                tip                          :tx_info_mock.tip                         .map_or(Operation::Retain, Operation::StartGlobal),
                paymaster_data               :tx_info_mock.paymaster_data              .map_or(Operation::Retain, Operation::StartGlobal),
                nonce_data_availability_mode :tx_info_mock.nonce_data_availability_mode.map_or(Operation::Retain, Operation::StartGlobal),
                fee_data_availability_mode   :tx_info_mock.fee_data_availability_mode  .map_or(Operation::Retain, Operation::StartGlobal),
                account_deployment_data      :tx_info_mock.account_deployment_data     .map_or(Operation::Retain, Operation::StartGlobal),
            },
            ..Default::default()
        });
    }

    pub fn start_spoof(&mut self, contract_address: ContractAddress, tx_info_mock: TxInfoMock) {
        self.spoof(contract_address, tx_info_mock, CheatSpan::Indefinite);
    }

    pub fn stop_spoof(&mut self, contract_address: ContractAddress) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            #[rustfmt::skip]
            tx_info: TxInfoMockOperations {
                version                      :Operation::Stop(contract_address),             
                account_contract_address     :Operation::Stop(contract_address),                            
                max_fee                      :Operation::Stop(contract_address),           
                signature                    :Operation::Stop(contract_address),             
                transaction_hash             :Operation::Stop(contract_address),                    
                chain_id                     :Operation::Stop(contract_address),            
                nonce                        :Operation::Stop(contract_address),         
                resource_bounds              :Operation::Stop(contract_address),                   
                tip                          :Operation::Stop(contract_address),       
                paymaster_data               :Operation::Stop(contract_address),                  
                nonce_data_availability_mode :Operation::Stop(contract_address),                                
                fee_data_availability_mode   :Operation::Stop(contract_address),                              
                account_deployment_data      :Operation::Stop(contract_address),                           
            },
            ..Default::default()
        });
    }
    pub fn stop_spoof_global(&mut self) {
        self.cheat_execution_info(ExecutionInfoMockOperations {
            #[rustfmt::skip]
            tx_info: TxInfoMockOperations {
                version                      :Operation::StopGlobal,             
                account_contract_address     :Operation::StopGlobal,                            
                max_fee                      :Operation::StopGlobal,           
                signature                    :Operation::StopGlobal,             
                transaction_hash             :Operation::StopGlobal,                    
                chain_id                     :Operation::StopGlobal,            
                nonce                        :Operation::StopGlobal,         
                resource_bounds              :Operation::StopGlobal,                   
                tip                          :Operation::StopGlobal,       
                paymaster_data               :Operation::StopGlobal,                  
                nonce_data_availability_mode :Operation::StopGlobal,                                
                fee_data_availability_mode   :Operation::StopGlobal,                              
                account_deployment_data      :Operation::StopGlobal,                           
            },
            ..Default::default()
        });
    }
}
