use crate::helpers::block_explorer::LinkProvider;
use crate::response::cast_message::SncastCommandMessage;
use crate::response::explorer_link::OutputLink;
use conversions::padded_felt::PaddedFelt;
use conversions::string::IntoDecStr;
use foundry_ui::styling::OutputBuilder;
use serde::{Serialize, Serializer};
use starknet_rust::core::types::{
    DataAvailabilityMode, DeclareTransaction, DeployAccountTransaction, InvokeTransaction,
    ResourceBoundsMapping, Transaction,
};
use starknet_types_core::felt::Felt;

#[derive(Clone)]
pub struct TransactionResponse(pub Transaction);

impl SncastCommandMessage for TransactionResponse {
    fn text(&self) -> String {
        match &self.0 {
            Transaction::Invoke(tx) => match tx {
                InvokeTransaction::V0(tx) => build_invoke_v0_response(tx),
                InvokeTransaction::V1(tx) => build_invoke_v1_response(tx),
                InvokeTransaction::V3(tx) => build_invoke_v3_response(tx),
            },
            Transaction::Declare(tx) => match tx {
                DeclareTransaction::V0(tx) => build_declare_v0_response(tx),
                DeclareTransaction::V1(tx) => build_declare_v1_response(tx),
                DeclareTransaction::V2(tx) => build_declare_v2_response(tx),
                DeclareTransaction::V3(tx) => build_declare_v3_response(tx),
            },
            Transaction::Deploy(tx) => build_deploy_response(tx),
            Transaction::DeployAccount(tx) => match tx {
                DeployAccountTransaction::V1(tx) => build_deploy_account_v1_response(tx),
                DeployAccountTransaction::V3(tx) => build_deploy_account_v3_response(tx),
            },
            Transaction::L1Handler(tx) => build_l1_handler_response(tx),
        }
    }
}

impl OutputLink for TransactionResponse {
    const TITLE: &'static str = "transaction";

    fn format_links(&self, provider: Box<dyn LinkProvider>) -> String {
        let hash = PaddedFelt(*self.0.transaction_hash());
        format!("transaction: {}", provider.transaction(hash))
    }
}

impl Serialize for TransactionResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Wrapper<'a> {
            transaction_type: &'static str,
            transaction: &'a Transaction,
        }

        Wrapper {
            transaction_type: json_transaction_type(&self.0),
            transaction: &self.0,
        }
        .serialize(serializer)
    }
}

fn json_transaction_type(tx: &Transaction) -> &'static str {
    match tx {
        Transaction::Invoke(InvokeTransaction::V0(_)) => "INVOKE_V0",
        Transaction::Invoke(InvokeTransaction::V1(_)) => "INVOKE_V1",
        Transaction::Invoke(InvokeTransaction::V3(_)) => "INVOKE_V3",
        Transaction::Declare(DeclareTransaction::V0(_)) => "DECLARE_V0",
        Transaction::Declare(DeclareTransaction::V1(_)) => "DECLARE_V1",
        Transaction::Declare(DeclareTransaction::V2(_)) => "DECLARE_V2",
        Transaction::Declare(DeclareTransaction::V3(_)) => "DECLARE_V3",
        Transaction::Deploy(_) => "DEPLOY",
        Transaction::DeployAccount(DeployAccountTransaction::V1(_)) => "DEPLOY_ACCOUNT_V1",
        Transaction::DeployAccount(DeployAccountTransaction::V3(_)) => "DEPLOY_ACCOUNT_V3",
        Transaction::L1Handler(_) => "L1_HANDLER",
    }
}

trait TransactionOutputBuilder {
    fn tx_header(self) -> Self;
    fn tx_type(self, tx_type: &str) -> Self;
    fn tx_version(self, version: &str) -> Self;
    fn tx_hash(self, hash: &Felt) -> Self;
    fn sender_address(self, addr: &Felt) -> Self;
    fn contract_address(self, addr: &Felt) -> Self;
    fn entry_point_selector(self, sel: &Felt) -> Self;
    fn class_hash(self, hash: &Felt) -> Self;
    fn compiled_class_hash(self, hash: &Felt) -> Self;
    fn contract_address_salt(self, salt: &Felt) -> Self;
    fn nonce(self, nonce: &Felt) -> Self;
    fn calldata(self, calldata: &[Felt]) -> Self;
    fn signature(self, sig: &[Felt]) -> Self;
    fn paymaster_data(self, data: &[Felt]) -> Self;
    fn account_deployment_data(self, data: &[Felt]) -> Self;
    fn constructor_calldata(self, data: &[Felt]) -> Self;
    fn resource_bounds(self, rb: &ResourceBoundsMapping) -> Self;
    fn max_fee(self, fee: &Felt) -> Self;
    fn tip(self, tip: u64) -> Self;
    fn nonce_da_mode(self, mode: DataAvailabilityMode) -> Self;
    fn fee_da_mode(self, mode: DataAvailabilityMode) -> Self;
    fn proof_facts(self, proof_facts: Option<&[Felt]>) -> Self;
}

impl TransactionOutputBuilder for OutputBuilder {
    fn tx_header(self) -> Self {
        self.success_message("Transaction found").blank_line()
    }
    fn tx_type(self, tx_type: &str) -> Self {
        self.field("Type", tx_type)
    }
    fn tx_version(self, version: &str) -> Self {
        self.field("Version", version)
    }
    fn tx_hash(self, hash: &Felt) -> Self {
        self.padded_felt_field("Transaction Hash", hash)
    }

    fn sender_address(self, addr: &Felt) -> Self {
        self.padded_felt_field("Sender Address", addr)
    }

    fn contract_address(self, addr: &Felt) -> Self {
        self.padded_felt_field("Contract Address", addr)
    }

    fn entry_point_selector(self, sel: &Felt) -> Self {
        self.padded_felt_field("Entry Point Selector", sel)
    }

    fn class_hash(self, hash: &Felt) -> Self {
        self.padded_felt_field("Class Hash", hash)
    }

    fn compiled_class_hash(self, hash: &Felt) -> Self {
        self.padded_felt_field("Compiled Class Hash", hash)
    }

    fn contract_address_salt(self, salt: &Felt) -> Self {
        self.padded_felt_field("Contract Address Salt", salt)
    }

    fn nonce(self, nonce: &Felt) -> Self {
        self.field("Nonce", &nonce.into_dec_string())
    }

    fn calldata(self, calldata: &[Felt]) -> Self {
        self.felt_list_field("Calldata", calldata)
    }

    fn signature(self, sig: &[Felt]) -> Self {
        self.felt_list_field("Signature", sig)
    }

    fn paymaster_data(self, data: &[Felt]) -> Self {
        self.felt_list_field("Paymaster Data", data)
    }

    fn account_deployment_data(self, data: &[Felt]) -> Self {
        self.felt_list_field("Account Deployment Data", data)
    }

    fn constructor_calldata(self, data: &[Felt]) -> Self {
        self.felt_list_field("Constructor Calldata", data)
    }

    fn resource_bounds(self, rb: &ResourceBoundsMapping) -> Self {
        self.field(
            "Resource Bounds L1 Gas",
            &format!(
                "max_amount={}, max_price_per_unit={}",
                rb.l1_gas.max_amount, rb.l1_gas.max_price_per_unit
            ),
        )
        .field(
            "Resource Bounds L1 Data Gas",
            &format!(
                "max_amount={}, max_price_per_unit={}",
                rb.l1_data_gas.max_amount, rb.l1_data_gas.max_price_per_unit
            ),
        )
        .field(
            "Resource Bounds L2 Gas",
            &format!(
                "max_amount={}, max_price_per_unit={}",
                rb.l2_gas.max_amount, rb.l2_gas.max_price_per_unit
            ),
        )
    }

    fn max_fee(self, fee: &Felt) -> Self {
        self.felt_field("Max Fee", fee)
    }

    fn tip(self, tip: u64) -> Self {
        self.field("Tip", &tip.to_string())
    }

    fn nonce_da_mode(self, mode: DataAvailabilityMode) -> Self {
        self.field("Nonce DA Mode", fmt_da(mode))
    }

    fn fee_da_mode(self, mode: DataAvailabilityMode) -> Self {
        self.field("Fee DA Mode", fmt_da(mode))
    }

    fn proof_facts(self, proof_facts: Option<&[Felt]>) -> Self {
        if let Some(proof_facts) = proof_facts {
            self.felt_list_field("Proof Facts", proof_facts)
        } else {
            self
        }
    }
}

fn build_invoke_v0_response(tx: &starknet_rust::core::types::InvokeTransactionV0) -> String {
    let starknet_rust::core::types::InvokeTransactionV0 {
        transaction_hash,
        max_fee,
        signature,
        contract_address,
        entry_point_selector,
        calldata,
    } = tx;
    OutputBuilder::new()
        .tx_header()
        .tx_type("INVOKE")
        .tx_version("0")
        .tx_hash(transaction_hash)
        .contract_address(contract_address)
        .entry_point_selector(entry_point_selector)
        .calldata(calldata)
        .max_fee(max_fee)
        .signature(signature)
        .build()
}

fn build_invoke_v1_response(tx: &starknet_rust::core::types::InvokeTransactionV1) -> String {
    let starknet_rust::core::types::InvokeTransactionV1 {
        transaction_hash,
        sender_address,
        calldata,
        max_fee,
        signature,
        nonce,
    } = tx;
    OutputBuilder::new()
        .tx_header()
        .tx_type("INVOKE")
        .tx_version("1")
        .tx_hash(transaction_hash)
        .sender_address(sender_address)
        .nonce(nonce)
        .calldata(calldata)
        .max_fee(max_fee)
        .signature(signature)
        .build()
}

fn build_invoke_v3_response(tx: &starknet_rust::core::types::InvokeTransactionV3) -> String {
    let starknet_rust::core::types::InvokeTransactionV3 {
        transaction_hash,
        sender_address,
        calldata,
        signature,
        nonce,
        resource_bounds,
        tip,
        paymaster_data,
        account_deployment_data,
        nonce_data_availability_mode,
        fee_data_availability_mode,
        proof_facts,
    } = tx;
    OutputBuilder::new()
        .tx_header()
        .tx_type("INVOKE")
        .tx_version("3")
        .tx_hash(transaction_hash)
        .sender_address(sender_address)
        .nonce(nonce)
        .calldata(calldata)
        .account_deployment_data(account_deployment_data)
        .resource_bounds(resource_bounds)
        .tip(*tip)
        .paymaster_data(paymaster_data)
        .nonce_da_mode(*nonce_data_availability_mode)
        .fee_da_mode(*fee_data_availability_mode)
        .signature(signature)
        .proof_facts(proof_facts.as_deref())
        .build()
}

fn build_declare_v0_response(tx: &starknet_rust::core::types::DeclareTransactionV0) -> String {
    let starknet_rust::core::types::DeclareTransactionV0 {
        transaction_hash,
        sender_address,
        max_fee,
        signature,
        class_hash,
    } = tx;
    OutputBuilder::new()
        .tx_header()
        .tx_type("DECLARE")
        .tx_version("0")
        .tx_hash(transaction_hash)
        .sender_address(sender_address)
        .class_hash(class_hash)
        .max_fee(max_fee)
        .signature(signature)
        .build()
}

fn build_declare_v1_response(tx: &starknet_rust::core::types::DeclareTransactionV1) -> String {
    let starknet_rust::core::types::DeclareTransactionV1 {
        transaction_hash,
        sender_address,
        max_fee,
        signature,
        nonce,
        class_hash,
    } = tx;
    OutputBuilder::new()
        .tx_header()
        .tx_type("DECLARE")
        .tx_version("1")
        .tx_hash(transaction_hash)
        .sender_address(sender_address)
        .nonce(nonce)
        .class_hash(class_hash)
        .max_fee(max_fee)
        .signature(signature)
        .build()
}

fn build_declare_v2_response(tx: &starknet_rust::core::types::DeclareTransactionV2) -> String {
    let starknet_rust::core::types::DeclareTransactionV2 {
        transaction_hash,
        sender_address,
        compiled_class_hash,
        max_fee,
        signature,
        nonce,
        class_hash,
    } = tx;
    OutputBuilder::new()
        .tx_header()
        .tx_type("DECLARE")
        .tx_version("2")
        .tx_hash(transaction_hash)
        .sender_address(sender_address)
        .nonce(nonce)
        .class_hash(class_hash)
        .compiled_class_hash(compiled_class_hash)
        .max_fee(max_fee)
        .signature(signature)
        .build()
}

fn build_declare_v3_response(tx: &starknet_rust::core::types::DeclareTransactionV3) -> String {
    let starknet_rust::core::types::DeclareTransactionV3 {
        transaction_hash,
        sender_address,
        compiled_class_hash,
        signature,
        nonce,
        class_hash,
        resource_bounds,
        tip,
        paymaster_data,
        account_deployment_data,
        nonce_data_availability_mode,
        fee_data_availability_mode,
    } = tx;
    OutputBuilder::new()
        .tx_header()
        .tx_type("DECLARE")
        .tx_version("3")
        .tx_hash(transaction_hash)
        .sender_address(sender_address)
        .nonce(nonce)
        .class_hash(class_hash)
        .compiled_class_hash(compiled_class_hash)
        .account_deployment_data(account_deployment_data)
        .resource_bounds(resource_bounds)
        .tip(*tip)
        .paymaster_data(paymaster_data)
        .nonce_da_mode(*nonce_data_availability_mode)
        .fee_da_mode(*fee_data_availability_mode)
        .signature(signature)
        .build()
}

fn build_deploy_response(tx: &starknet_rust::core::types::DeployTransaction) -> String {
    let starknet_rust::core::types::DeployTransaction {
        transaction_hash,
        version,
        contract_address_salt,
        constructor_calldata,
        class_hash,
    } = tx;
    OutputBuilder::new()
        .tx_header()
        .tx_type("DEPLOY")
        .tx_version(&version.to_string())
        .tx_hash(transaction_hash)
        .class_hash(class_hash)
        .contract_address_salt(contract_address_salt)
        .constructor_calldata(constructor_calldata)
        .build()
}

fn build_deploy_account_v1_response(
    tx: &starknet_rust::core::types::DeployAccountTransactionV1,
) -> String {
    let starknet_rust::core::types::DeployAccountTransactionV1 {
        transaction_hash,
        max_fee,
        signature,
        nonce,
        contract_address_salt,
        constructor_calldata,
        class_hash,
    } = tx;
    OutputBuilder::new()
        .tx_header()
        .tx_type("DEPLOY ACCOUNT")
        .tx_version("1")
        .tx_hash(transaction_hash)
        .nonce(nonce)
        .class_hash(class_hash)
        .contract_address_salt(contract_address_salt)
        .constructor_calldata(constructor_calldata)
        .max_fee(max_fee)
        .signature(signature)
        .build()
}

fn build_deploy_account_v3_response(
    tx: &starknet_rust::core::types::DeployAccountTransactionV3,
) -> String {
    let starknet_rust::core::types::DeployAccountTransactionV3 {
        transaction_hash,
        signature,
        nonce,
        contract_address_salt,
        constructor_calldata,
        class_hash,
        resource_bounds,
        tip,
        paymaster_data,
        nonce_data_availability_mode,
        fee_data_availability_mode,
    } = tx;
    OutputBuilder::new()
        .tx_header()
        .tx_type("DEPLOY ACCOUNT")
        .tx_version("3")
        .tx_hash(transaction_hash)
        .nonce(nonce)
        .class_hash(class_hash)
        .contract_address_salt(contract_address_salt)
        .constructor_calldata(constructor_calldata)
        .resource_bounds(resource_bounds)
        .tip(*tip)
        .paymaster_data(paymaster_data)
        .nonce_da_mode(*nonce_data_availability_mode)
        .fee_da_mode(*fee_data_availability_mode)
        .signature(signature)
        .build()
}

fn build_l1_handler_response(tx: &starknet_rust::core::types::L1HandlerTransaction) -> String {
    let starknet_rust::core::types::L1HandlerTransaction {
        transaction_hash,
        version,
        nonce,
        contract_address,
        entry_point_selector,
        calldata,
    } = tx;
    OutputBuilder::new()
        .tx_header()
        .tx_type("L1 HANDLER")
        .tx_version(&version.to_string())
        .tx_hash(transaction_hash)
        .contract_address(contract_address)
        .nonce(&Felt::from(*nonce))
        .entry_point_selector(entry_point_selector)
        .calldata(calldata)
        .build()
}

fn fmt_da(mode: DataAvailabilityMode) -> &'static str {
    match mode {
        DataAvailabilityMode::L1 => "L1",
        DataAvailabilityMode::L2 => "L2",
    }
}
