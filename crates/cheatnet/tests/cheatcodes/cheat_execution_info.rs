use super::test_environment::TestEnvironment;
use crate::common::{assertions::assert_success, get_contracts, recover_data};
use cairo_vm::Felt252;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::cheat_execution_info::ResourceBounds;
use cheatnet::runtime_extensions::forge_runtime_extension::cheatcodes::cheat_execution_info::{
    CheatArguments, ExecutionInfoMockOperations, Operation, TxInfoMockOperations,
};
use cheatnet::state::CheatSpan;
use conversions::serde::deserialize::{BufferReader, CairoDeserialize};
use conversions::IntoConv;
use starknet_api::{core::ContractAddress, transaction::TransactionHash};

trait CheatTransactionHashTrait {
    fn cheat_transaction_hash(
        &mut self,
        contract_address: ContractAddress,
        transaction_hash: Felt252,
        span: CheatSpan,
    );
    fn start_cheat_transaction_hash(
        &mut self,
        contract_address: ContractAddress,
        transaction_hash: Felt252,
    );
    fn stop_cheat_transaction_hash(&mut self, contract_address: ContractAddress);
    fn start_cheat_transaction_hash_global(&mut self, transaction_hash: Felt252);
    fn stop_cheat_transaction_hash_global(&mut self);
}
impl CheatTransactionHashTrait for TestEnvironment {
    fn cheat_transaction_hash(
        &mut self,
        contract_address: ContractAddress,
        transaction_hash: Felt252,
        span: CheatSpan,
    ) {
        let mut execution_info_mock = ExecutionInfoMockOperations::default();

        execution_info_mock.tx_info.transaction_hash = Operation::Start(CheatArguments {
            value: transaction_hash,
            span,
            target: contract_address,
        });

        self.cheatnet_state
            .cheat_execution_info(execution_info_mock);
    }

    fn start_cheat_transaction_hash(
        &mut self,
        contract_address: ContractAddress,
        transaction_hash: Felt252,
    ) {
        let mut execution_info_mock = ExecutionInfoMockOperations::default();

        execution_info_mock.tx_info.transaction_hash = Operation::Start(CheatArguments {
            value: transaction_hash,
            span: CheatSpan::Indefinite,
            target: contract_address,
        });

        self.cheatnet_state
            .cheat_execution_info(execution_info_mock);
    }

    fn stop_cheat_transaction_hash(&mut self, contract_address: ContractAddress) {
        let mut execution_info_mock = ExecutionInfoMockOperations::default();

        execution_info_mock.tx_info.transaction_hash = Operation::Stop(contract_address);

        self.cheatnet_state
            .cheat_execution_info(execution_info_mock);
    }

    fn start_cheat_transaction_hash_global(&mut self, transaction_hash: Felt252) {
        let mut execution_info_mock = ExecutionInfoMockOperations::default();

        execution_info_mock.tx_info.transaction_hash = Operation::StartGlobal(transaction_hash);

        self.cheatnet_state
            .cheat_execution_info(execution_info_mock);
    }
    fn stop_cheat_transaction_hash_global(&mut self) {
        let mut execution_info_mock = ExecutionInfoMockOperations::default();

        execution_info_mock.tx_info.transaction_hash = Operation::StopGlobal;

        self.cheatnet_state
            .cheat_execution_info(execution_info_mock);
    }
}

trait CheatTransactionInfoTrait {
    fn cheat_transaction_info(&mut self, tx_info_mock: TxInfoMockOperations);
}

impl CheatTransactionInfoTrait for TestEnvironment {
    fn cheat_transaction_info(&mut self, tx_info_mock: TxInfoMockOperations) {
        let execution_info_mock_operations = ExecutionInfoMockOperations {
            tx_info: tx_info_mock,
            ..Default::default()
        };

        self.cheatnet_state
            .cheat_execution_info(execution_info_mock_operations);
    }
}

trait TxInfoTrait {
    fn assert_tx_info(&mut self, contract_address: &ContractAddress, expected_tx_info: &TxInfo);
    fn get_tx_info(&mut self, contract_address: &ContractAddress) -> TxInfo;
}

impl TxInfoTrait for TestEnvironment {
    fn assert_tx_info(&mut self, contract_address: &ContractAddress, expected_tx_info: &TxInfo) {
        let tx_info = self.get_tx_info(contract_address);
        assert_eq!(tx_info, *expected_tx_info);
    }

    fn get_tx_info(&mut self, contract_address: &ContractAddress) -> TxInfo {
        let call_result = self.call_contract(contract_address, "get_tx_info", &[]);
        let data = recover_data(call_result);
        TxInfo::deserialize(&data)
    }
}

#[derive(CairoDeserialize, Clone, Default, Debug, PartialEq)]
struct TxInfo {
    pub version: Felt252,
    pub account_contract_address: Felt252,
    pub max_fee: Felt252,
    pub signature: Vec<Felt252>,
    pub transaction_hash: Felt252,
    pub chain_id: Felt252,
    pub nonce: Felt252,
    pub resource_bounds: Vec<ResourceBounds>,
    pub tip: Felt252,
    pub paymaster_data: Vec<Felt252>,
    pub nonce_data_availability_mode: Felt252,
    pub fee_data_availability_mode: Felt252,
    pub account_deployment_data: Vec<Felt252>,
}

impl TxInfo {
    fn apply_mock_fields(tx_info_mock: &TxInfoMockOperations, tx_info: &Self) -> Self {
        macro_rules! clone_field {
            ($field:ident) => {
                if let Operation::Start(CheatArguments {
                    value,
                    span: CheatSpan::Indefinite,
                    target: _contract_address,
                }) = tx_info_mock.$field.clone()
                {
                    value
                } else {
                    tx_info.$field.clone()
                }
            };
        }

        Self {
            version: clone_field!(version),
            account_contract_address: clone_field!(account_contract_address),
            max_fee: clone_field!(max_fee),
            signature: clone_field!(signature),
            transaction_hash: clone_field!(transaction_hash),
            chain_id: clone_field!(chain_id),
            nonce: clone_field!(nonce),
            resource_bounds: clone_field!(resource_bounds),
            tip: clone_field!(tip),
            paymaster_data: clone_field!(paymaster_data),
            nonce_data_availability_mode: clone_field!(nonce_data_availability_mode),
            fee_data_availability_mode: clone_field!(fee_data_availability_mode),
            account_deployment_data: clone_field!(account_deployment_data),
        }
    }

    fn deserialize(data: &[Felt252]) -> Self {
        BufferReader::new(data).read().unwrap()
    }
}

#[test]
fn cheat_transaction_hash_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatTxInfoChecker", &[]);

    let tx_info_before = test_env.get_tx_info(&contract_address);

    let transaction_hash = Felt252::from(123);
    let mut expected_tx_info = tx_info_before.clone();

    expected_tx_info.transaction_hash = transaction_hash;

    test_env.start_cheat_transaction_hash(contract_address, transaction_hash);

    test_env.assert_tx_info(&contract_address, &expected_tx_info);
}

#[test]
fn start_cheat_execution_info_multiple_times() {
    fn operation_start<T>(contract_address: ContractAddress, value: T) -> Operation<T> {
        Operation::Start(CheatArguments {
            value,
            span: CheatSpan::Indefinite,
            target: contract_address,
        })
    }

    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatTxInfoChecker", &[]);

    let tx_info_before = test_env.get_tx_info(&contract_address);

    let initial_tx_info_mock = TxInfoMockOperations {
        version: operation_start(contract_address, Felt252::from(13)),
        account_contract_address: operation_start(contract_address, Felt252::from(66)),
        max_fee: operation_start(contract_address, Felt252::from(77)),
        signature: operation_start(contract_address, vec![Felt252::from(88), Felt252::from(89)]),
        transaction_hash: operation_start(contract_address, Felt252::from(123)),
        chain_id: operation_start(contract_address, Felt252::from(22)),
        nonce: operation_start(contract_address, Felt252::from(33)),
        resource_bounds: operation_start(
            contract_address,
            vec![
                ResourceBounds {
                    resource: Felt252::from(111),
                    max_amount: 222,
                    max_price_per_unit: 333,
                },
                ResourceBounds {
                    resource: Felt252::from(444),
                    max_amount: 555,
                    max_price_per_unit: 666,
                },
            ],
        ),
        tip: operation_start(contract_address, Felt252::from(777)),
        paymaster_data: operation_start(
            contract_address,
            vec![
                Felt252::from(11),
                Felt252::from(22),
                Felt252::from(33),
                Felt252::from(44),
            ],
        ),
        nonce_data_availability_mode: operation_start(contract_address, Felt252::from(55)),
        fee_data_availability_mode: operation_start(contract_address, Felt252::from(66)),
        account_deployment_data: operation_start(
            contract_address,
            vec![Felt252::from(777), Felt252::from(888), Felt252::from(999)],
        ),
    };

    let expected_tx_info = TxInfo::apply_mock_fields(&initial_tx_info_mock, &tx_info_before);

    test_env.cheat_transaction_info(initial_tx_info_mock.clone());

    test_env.assert_tx_info(&contract_address, &expected_tx_info);

    let tx_info_mock = TxInfoMockOperations {
        version: Operation::Retain,
        max_fee: Operation::Retain,
        transaction_hash: Operation::Retain,
        nonce: Operation::Retain,
        tip: Operation::Retain,
        nonce_data_availability_mode: Operation::Retain,
        account_deployment_data: Operation::Retain,
        ..initial_tx_info_mock
    };

    let expected_tx_info = TxInfo::apply_mock_fields(&tx_info_mock, &expected_tx_info);

    test_env.cheat_transaction_info(tx_info_mock);

    test_env.assert_tx_info(&contract_address, &expected_tx_info);

    let tx_info_mock = TxInfoMockOperations {
        account_contract_address: Operation::Retain,
        signature: Operation::Retain,
        chain_id: Operation::Retain,
        resource_bounds: Operation::Retain,
        paymaster_data: Operation::Retain,
        fee_data_availability_mode: Operation::Retain,
        ..initial_tx_info_mock
    };

    let expected_tx_info = TxInfo::apply_mock_fields(&tx_info_mock, &expected_tx_info);

    test_env.cheat_transaction_info(tx_info_mock);

    test_env.assert_tx_info(&contract_address, &expected_tx_info);
}

#[test]
fn cheat_transaction_hash_start_stop() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatTxInfoChecker", &[]);

    let tx_info_before = test_env.get_tx_info(&contract_address);

    let transaction_hash = Felt252::from(123);
    let mut expected_tx_info = tx_info_before.clone();

    expected_tx_info.transaction_hash = transaction_hash;

    test_env.start_cheat_transaction_hash(contract_address, transaction_hash);

    test_env.assert_tx_info(&contract_address, &expected_tx_info);

    test_env.stop_cheat_transaction_hash(contract_address);

    test_env.assert_tx_info(&contract_address, &tx_info_before);
}

#[test]
fn cheat_transaction_hash_stop_no_effect() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatTxInfoChecker", &[]);

    let tx_info_before = test_env.get_tx_info(&contract_address);

    test_env.stop_cheat_transaction_hash(contract_address);

    test_env.assert_tx_info(&contract_address, &tx_info_before);
}

#[test]
fn cheat_transaction_hash_with_other_syscall() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatTxInfoChecker", &[]);

    let transaction_hash = Felt252::from(123);

    test_env.start_cheat_transaction_hash(contract_address, transaction_hash);

    let output = test_env.call_contract(&contract_address, "get_tx_hash_and_emit_event", &[]);

    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn cheat_transaction_hash_in_constructor() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("TxHashChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    let transaction_hash = Felt252::from(123);

    test_env.start_cheat_transaction_hash(precalculated_address, transaction_hash);

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);

    assert_eq!(precalculated_address, contract_address);

    let output = test_env.call_contract(&contract_address, "get_stored_tx_hash", &[]);
    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn cheat_transaction_hash_proxy() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatTxInfoChecker", &[]);

    let transaction_hash = Felt252::from(123);

    test_env.start_cheat_transaction_hash(contract_address, transaction_hash);

    let output = test_env.call_contract(&contract_address, "get_transaction_hash", &[]);
    assert_success(output, &[Felt252::from(123)]);

    let proxy_address = test_env.deploy("TxHashCheckerProxy", &[]);

    let output = test_env.call_contract(
        &proxy_address,
        "get_checkers_tx_hash",
        &[contract_address.into_()],
    );

    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn cheat_transaction_hash_library_call() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatTxInfoChecker", &contracts_data);

    let lib_call_address = test_env.deploy("CheatTxInfoCheckerLibCall", &[]);

    let transaction_hash = Felt252::from(123);

    test_env.start_cheat_transaction_hash(lib_call_address, transaction_hash);

    let output = test_env.call_contract(
        &lib_call_address,
        "get_tx_hash_with_lib_call",
        &[class_hash.into_()],
    );

    assert_success(output, &[Felt252::from(123)]);
}

#[test]
fn cheat_transaction_hash_all_simple() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatTxInfoChecker", &[]);

    let tx_info_before = test_env.get_tx_info(&contract_address);

    let transaction_hash = Felt252::from(123);
    let mut expected_tx_info = tx_info_before.clone();

    expected_tx_info.transaction_hash = transaction_hash;

    test_env.start_cheat_transaction_hash_global(transaction_hash);

    test_env.assert_tx_info(&contract_address, &expected_tx_info);
}

#[test]
fn cheat_transaction_hash_all_then_one() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatTxInfoChecker", &[]);

    let tx_info_before = test_env.get_tx_info(&contract_address);

    let transaction_hash = Felt252::from(321);
    let mut expected_tx_info = tx_info_before.clone();

    expected_tx_info.transaction_hash = transaction_hash;

    test_env.start_cheat_transaction_hash_global(Felt252::from(123));

    test_env.start_cheat_transaction_hash(contract_address, transaction_hash);

    test_env.assert_tx_info(&contract_address, &expected_tx_info);
}

#[test]
fn cheat_transaction_hash_one_then_all() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatTxInfoChecker", &[]);

    let tx_info_before = test_env.get_tx_info(&contract_address);

    let transaction_hash = Felt252::from(321);
    let mut expected_tx_info = tx_info_before.clone();

    expected_tx_info.transaction_hash = transaction_hash;

    test_env.start_cheat_transaction_hash(contract_address, Felt252::from(123));

    test_env.start_cheat_transaction_hash_global(transaction_hash);

    test_env.assert_tx_info(&contract_address, &expected_tx_info);
}

#[test]
fn cheat_transaction_hash_all_stop() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatTxInfoChecker", &[]);

    let tx_info_before = test_env.get_tx_info(&contract_address);

    let transaction_hash = Felt252::from(123);
    let expected_tx_info = tx_info_before.clone();

    test_env.start_cheat_transaction_hash_global(transaction_hash);

    test_env.stop_cheat_transaction_hash_global();

    test_env.assert_tx_info(&contract_address, &expected_tx_info);
}

#[test]
fn cheat_transaction_hash_multiple() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatTxInfoChecker", &contracts_data);

    let contract_address_1 = test_env.deploy_wrapper(&class_hash, &[]);

    let contract_address_2 = test_env.deploy_wrapper(&class_hash, &[]);

    let tx_info_before_1 = test_env.get_tx_info(&contract_address_1);
    let tx_info_before_2 = test_env.get_tx_info(&contract_address_2);

    let transaction_hash = Felt252::from(123);
    let mut expected_tx_info_1 = tx_info_before_1.clone();
    let mut expected_tx_info_2 = tx_info_before_2.clone();

    expected_tx_info_1.transaction_hash = transaction_hash;

    expected_tx_info_2.transaction_hash = transaction_hash;

    test_env.start_cheat_transaction_hash(contract_address_1, transaction_hash);
    test_env.start_cheat_transaction_hash(contract_address_2, transaction_hash);

    test_env.assert_tx_info(&contract_address_1, &expected_tx_info_1);
    test_env.assert_tx_info(&contract_address_2, &expected_tx_info_2);

    test_env.stop_cheat_transaction_hash_global();

    test_env.assert_tx_info(&contract_address_1, &tx_info_before_1);
    test_env.assert_tx_info(&contract_address_2, &tx_info_before_2);
}

#[test]
fn cheat_transaction_hash_simple_with_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatTxInfoChecker", &[]);

    let tx_info_before = test_env.get_tx_info(&contract_address);
    let transaction_hash = Felt252::from(123);

    let mut expected_tx_info = tx_info_before.clone();
    expected_tx_info.transaction_hash = transaction_hash;

    test_env.cheat_transaction_hash(
        contract_address,
        transaction_hash,
        CheatSpan::TargetCalls(2),
    );

    test_env.assert_tx_info(&contract_address, &expected_tx_info);
    test_env.assert_tx_info(&contract_address, &expected_tx_info);
    test_env.assert_tx_info(&contract_address, &tx_info_before);
}

#[test]
fn cheat_transaction_hash_proxy_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("TxHashCheckerProxy", &contracts_data);
    let contract_address_1 = test_env.deploy_wrapper(&class_hash, &[]);
    let contract_address_2 = test_env.deploy_wrapper(&class_hash, &[]);

    test_env.cheat_transaction_hash(
        contract_address_1,
        Felt252::from(123),
        CheatSpan::TargetCalls(1),
    );

    let output = test_env.call_contract(
        &contract_address_1,
        "call_proxy",
        &[contract_address_2.into_()],
    );
    assert_success(output, &[123.into(), TransactionHash::default().0.into_()]);
}

#[test]
fn cheat_transaction_hash_in_constructor_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("TxHashChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.cheat_transaction_hash(
        precalculated_address,
        Felt252::from(123),
        CheatSpan::TargetCalls(2),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_transaction_hash", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_transaction_hash", &[]),
        &[TransactionHash::default().0.into_()],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_stored_tx_hash", &[]),
        &[Felt252::from(123)],
    );
}

#[test]
fn cheat_transaction_hash_no_constructor_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();

    let class_hash = test_env.declare("CheatTxInfoChecker", &contracts_data);
    let precalculated_address = test_env.precalculate_address(&class_hash, &[]);

    test_env.cheat_transaction_hash(
        precalculated_address,
        Felt252::from(123),
        CheatSpan::TargetCalls(1),
    );

    let contract_address = test_env.deploy_wrapper(&class_hash, &[]);
    assert_eq!(precalculated_address, contract_address);

    assert_success(
        test_env.call_contract(&contract_address, "get_transaction_hash", &[]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, "get_transaction_hash", &[]),
        &[TransactionHash::default().0.into_()],
    );
}

#[test]
fn cheat_transaction_hash_override_span() {
    let mut test_env = TestEnvironment::new();

    let contract_address = test_env.deploy("CheatTxInfoChecker", &[]);

    let tx_info_before = test_env.get_tx_info(&contract_address);
    let transaction_hash = Felt252::from(123);

    let mut expected_tx_info = tx_info_before.clone();
    expected_tx_info.transaction_hash = transaction_hash;

    test_env.cheat_transaction_hash(contract_address, transaction_hash, CheatSpan::Indefinite);

    test_env.assert_tx_info(&contract_address, &expected_tx_info);

    let transaction_hash = Felt252::from(321);

    expected_tx_info.transaction_hash = transaction_hash;

    test_env.cheat_transaction_hash(
        contract_address,
        transaction_hash,
        CheatSpan::TargetCalls(1),
    );

    test_env.assert_tx_info(&contract_address, &expected_tx_info);
    test_env.assert_tx_info(&contract_address, &tx_info_before);
}

#[test]
fn cheat_transaction_hash_library_call_with_span() {
    let mut test_env = TestEnvironment::new();

    let contracts_data = get_contracts();
    let class_hash = test_env.declare("CheatTxInfoChecker", &contracts_data);
    let contract_address = test_env.deploy("CheatTxInfoCheckerLibCall", &[]);

    let tx_info_before = test_env.get_tx_info(&contract_address);

    test_env.cheat_transaction_hash(
        contract_address,
        Felt252::from(123),
        CheatSpan::TargetCalls(1),
    );

    let lib_call_selector = "get_tx_hash_with_lib_call";

    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[Felt252::from(123)],
    );
    assert_success(
        test_env.call_contract(&contract_address, lib_call_selector, &[class_hash.into_()]),
        &[tx_info_before.transaction_hash],
    );
}
