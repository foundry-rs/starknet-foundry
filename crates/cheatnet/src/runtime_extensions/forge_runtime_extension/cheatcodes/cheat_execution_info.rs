use crate::{
    state::{CheatSpan, CheatStatus},
    CheatnetState,
};
use conversions::serde::{deserialize::CairoDeserialize, serialize::CairoSerialize};
use starknet_api::core::ContractAddress;
use starknet_types_core::felt::Felt;

#[derive(CairoDeserialize, Clone, Debug)]
pub struct CheatArguments<T> {
    pub value: T,
    pub span: CheatSpan,
    pub target: ContractAddress,
}

#[derive(CairoDeserialize, Clone, Default, Debug)]
pub enum Operation<T> {
    StartGlobal(T),
    Start(CheatArguments<T>),
    Stop(ContractAddress),
    StopGlobal,
    #[default]
    Retain,
}

#[derive(CairoDeserialize, CairoSerialize, Clone, Default, Debug, Eq, PartialEq)]
pub struct ResourceBounds {
    pub resource: Felt,
    pub max_amount: u64,
    pub max_price_per_unit: u128,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct TxInfoMock {
    pub version: CheatStatus<Felt>,
    pub account_contract_address: CheatStatus<Felt>,
    pub max_fee: CheatStatus<Felt>,
    pub signature: CheatStatus<Vec<Felt>>,
    pub transaction_hash: CheatStatus<Felt>,
    pub chain_id: CheatStatus<Felt>,
    pub nonce: CheatStatus<Felt>,
    pub resource_bounds: CheatStatus<Vec<ResourceBounds>>,
    pub tip: CheatStatus<Felt>,
    pub paymaster_data: CheatStatus<Vec<Felt>>,
    pub nonce_data_availability_mode: CheatStatus<Felt>,
    pub fee_data_availability_mode: CheatStatus<Felt>,
    pub account_deployment_data: CheatStatus<Vec<Felt>>,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct BlockInfoMock {
    pub block_number: CheatStatus<u64>,
    pub block_timestamp: CheatStatus<u64>,
    pub sequencer_address: CheatStatus<ContractAddress>,
}

#[derive(Clone, Default, Debug)]
pub struct ExecutionInfoMock {
    pub block_info: BlockInfoMock,
    pub tx_info: TxInfoMock,
    pub caller_address: CheatStatus<ContractAddress>,
}

#[derive(CairoDeserialize, Clone, Default, Debug)]
pub struct TxInfoMockOperations {
    pub version: Operation<Felt>,
    pub account_contract_address: Operation<Felt>,
    pub max_fee: Operation<Felt>,
    pub signature: Operation<Vec<Felt>>,
    pub transaction_hash: Operation<Felt>,
    pub chain_id: Operation<Felt>,
    pub nonce: Operation<Felt>,
    pub resource_bounds: Operation<Vec<ResourceBounds>>,
    pub tip: Operation<Felt>,
    pub paymaster_data: Operation<Vec<Felt>>,
    pub nonce_data_availability_mode: Operation<Felt>,
    pub fee_data_availability_mode: Operation<Felt>,
    pub account_deployment_data: Operation<Vec<Felt>>,
}

#[derive(CairoDeserialize, Clone, Default, Debug)]
pub struct BlockInfoMockOperations {
    pub block_number: Operation<u64>,
    pub block_timestamp: Operation<u64>,
    pub sequencer_address: Operation<ContractAddress>,
}

#[derive(CairoDeserialize, Clone, Default, Debug)]
pub struct ExecutionInfoMockOperations {
    pub block_info: BlockInfoMockOperations,
    pub tx_info: TxInfoMockOperations,
    pub caller_address: Operation<ContractAddress>,
}

macro_rules! for_all_fields {
    ($macro:ident!) => {
        $macro!(caller_address);

        $macro!(block_info.block_number);
        $macro!(block_info.block_timestamp);
        $macro!(block_info.sequencer_address);

        $macro!(tx_info.version);
        $macro!(tx_info.account_contract_address);
        $macro!(tx_info.max_fee);
        $macro!(tx_info.signature);
        $macro!(tx_info.transaction_hash);
        $macro!(tx_info.chain_id);
        $macro!(tx_info.nonce);
        $macro!(tx_info.resource_bounds);
        $macro!(tx_info.tip);
        $macro!(tx_info.paymaster_data);
        $macro!(tx_info.nonce_data_availability_mode);
        $macro!(tx_info.fee_data_availability_mode);
        $macro!(tx_info.account_deployment_data);
    };
}

impl CheatnetState {
    pub fn get_cheated_execution_info_for_contract(
        &mut self,
        target: ContractAddress,
    ) -> &mut ExecutionInfoMock {
        self.cheated_execution_info_contracts
            .entry(target)
            .or_insert_with(|| self.global_cheated_execution_info.clone())
    }

    pub fn cheat_execution_info(&mut self, execution_info_mock: ExecutionInfoMockOperations) {
        macro_rules! cheat {
            ($($path:ident).+) => {
                match execution_info_mock.$($path).+ {
                    Operation::Retain => {}
                    Operation::Start(CheatArguments {
                        value,
                        span,
                        target,
                    }) => {
                        let cheated_info = self.get_cheated_execution_info_for_contract(target);

                        cheated_info.$($path).+ = CheatStatus::Cheated(value, span);
                    }
                    Operation::Stop(target) => {
                        let cheated_info = self.get_cheated_execution_info_for_contract(target);

                        cheated_info.$($path).+ = CheatStatus::Uncheated;
                    }
                    Operation::StartGlobal(value) => {
                        self.global_cheated_execution_info.$($path).+ =
                            CheatStatus::Cheated(value.clone(), CheatSpan::Indefinite);

                        for val in self.cheated_execution_info_contracts.values_mut() {
                            val.$($path).+ = CheatStatus::Cheated(value.clone(), CheatSpan::Indefinite);
                        }
                    }
                    Operation::StopGlobal => {
                        self.global_cheated_execution_info.$($path).+ = CheatStatus::Uncheated;

                        for val in self.cheated_execution_info_contracts.values_mut() {
                            val.$($path).+ = CheatStatus::Uncheated;
                        }
                    }
                };
            };
        }

        for_all_fields!(cheat!);
    }

    pub fn progress_cheated_execution_info(&mut self, address: ContractAddress) {
        let mocks = self.get_cheated_execution_info_for_contract(address);

        macro_rules! decrement {
            ($($path:ident).+) => {
                mocks.$($path).+.decrement_cheat_span();
            };
        }

        for_all_fields!(decrement!);
    }
}