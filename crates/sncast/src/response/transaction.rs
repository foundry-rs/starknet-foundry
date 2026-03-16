use crate::helpers::block_explorer::LinkProvider;
use crate::response::cast_message::SncastCommandMessage;
use crate::response::explorer_link::OutputLink;
use conversions::padded_felt::PaddedFelt;
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
        let (tx_type, tx_version) = tx_type_and_version(&self.0);
        match &self.0 {
            Transaction::Invoke(tx) => match tx {
                InvokeTransaction::V0(tx) => fmt_invoke_v0(tx, tx_type, tx_version),
                InvokeTransaction::V1(tx) => fmt_invoke_v1(tx, tx_type, tx_version),
                InvokeTransaction::V3(tx) => fmt_invoke_v3(tx, tx_type, tx_version),
            },
            Transaction::Declare(tx) => match tx {
                DeclareTransaction::V0(tx) => fmt_declare_v0(tx, tx_type, tx_version),
                DeclareTransaction::V1(tx) => fmt_declare_v1(tx, tx_type, tx_version),
                DeclareTransaction::V2(tx) => fmt_declare_v2(tx, tx_type, tx_version),
                DeclareTransaction::V3(tx) => fmt_declare_v3(tx, tx_type, tx_version),
            },
            Transaction::Deploy(tx) => fmt_deploy(tx, tx_type, tx_version),
            Transaction::DeployAccount(tx) => match tx {
                DeployAccountTransaction::V1(tx) => fmt_deploy_account_v1(tx, tx_type, tx_version),
                DeployAccountTransaction::V3(tx) => fmt_deploy_account_v3(tx, tx_type, tx_version),
            },
            Transaction::L1Handler(tx) => fmt_l1_handler(tx, tx_type, tx_version),
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

fn tx_type_and_version(tx: &Transaction) -> (&'static str, Option<&'static str>) {
    match tx {
        Transaction::Invoke(InvokeTransaction::V0(_)) => ("Invoke", Some("V0")),
        Transaction::Invoke(InvokeTransaction::V1(_)) => ("Invoke", Some("V1")),
        Transaction::Invoke(InvokeTransaction::V3(_)) => ("Invoke", Some("V3")),
        Transaction::Declare(DeclareTransaction::V0(_)) => ("Declare", Some("V0")),
        Transaction::Declare(DeclareTransaction::V1(_)) => ("Declare", Some("V1")),
        Transaction::Declare(DeclareTransaction::V2(_)) => ("Declare", Some("V2")),
        Transaction::Declare(DeclareTransaction::V3(_)) => ("Declare", Some("V3")),
        Transaction::Deploy(_) => ("Deploy", None),
        Transaction::DeployAccount(DeployAccountTransaction::V1(_)) => {
            ("Deploy Account", Some("V1"))
        }
        Transaction::DeployAccount(DeployAccountTransaction::V3(_)) => {
            ("Deploy Account", Some("V3"))
        }
        Transaction::L1Handler(_) => ("L1 Handler", None),
    }
}

impl Serialize for TransactionResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Wrapper<'a> {
            transaction_type: &'a str,
            transaction: &'a Transaction,
        }

        let (name, version) = tx_type_and_version(&self.0);
        let tag = match version {
            Some(v) => format!("{name}_{v}"),
            None => name.to_string(),
        };

        Wrapper {
            transaction_type: &tag,
            transaction: &self.0,
        }
        .serialize(serializer)
    }
}

trait TransactionOutputBuilder {
    fn transaction_hash(self, hash: &Felt) -> Self;
    fn sender_address(self, addr: &Felt) -> Self;
    fn contract_address(self, addr: &Felt) -> Self;
    fn entry_point_selector(self, sel: &Felt) -> Self;
    fn class_hash(self, hash: &Felt) -> Self;
    fn compiled_class_hash(self, hash: &Felt) -> Self;
    fn contract_address_salt(self, salt: &Felt) -> Self;
    fn transaction_nonce(self, nonce: &Felt) -> Self;
    fn transaction_version(self, version: &Felt) -> Self;
    fn calldata(self, calldata: &[Felt]) -> Self;
    fn signature(self, sig: &[Felt]) -> Self;
    fn paymaster_data(self, data: &[Felt]) -> Self;
    fn account_deployment_data(self, data: &[Felt]) -> Self;
    fn constructor_calldata(self, data: &[Felt]) -> Self;
    fn resource_bounds(self, rb: &ResourceBoundsMapping) -> Self;
    fn max_fee(self, fee: &Felt) -> Self;
    fn tip(self, tip: u64) -> Self;
    fn nonce_da_mode(self, mode: &DataAvailabilityMode) -> Self;
    fn fee_da_mode(self, mode: &DataAvailabilityMode) -> Self;
    fn felt_field(self, label: &str, felt: &Felt) -> Self;
    fn short_felt_field(self, label: &str, felt: &Felt) -> Self;
    fn felt_list_field(self, label: &str, felts: &[Felt]) -> Self;
}

impl TransactionOutputBuilder for OutputBuilder {
    fn transaction_hash(self, hash: &Felt) -> Self {
        self.felt_field("Transaction Hash", hash)
    }

    fn sender_address(self, addr: &Felt) -> Self {
        self.felt_field("Sender Address", addr)
    }

    fn contract_address(self, addr: &Felt) -> Self {
        self.felt_field("Contract Address", addr)
    }

    fn entry_point_selector(self, sel: &Felt) -> Self {
        self.felt_field("Entry Point Selector", sel)
    }

    fn class_hash(self, hash: &Felt) -> Self {
        self.felt_field("Class Hash", hash)
    }

    fn compiled_class_hash(self, hash: &Felt) -> Self {
        self.felt_field("Compiled Class Hash", hash)
    }

    fn contract_address_salt(self, salt: &Felt) -> Self {
        self.felt_field("Contract Address Salt", salt)
    }

    fn transaction_nonce(self, nonce: &Felt) -> Self {
        self.short_felt_field("Nonce", nonce)
    }

    fn transaction_version(self, version: &Felt) -> Self {
        self.short_felt_field("Version", version)
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
                "max_amount={:#x}, max_price={:#x}",
                rb.l1_gas.max_amount, rb.l1_gas.max_price_per_unit
            ),
        )
        .field(
            "Resource Bounds L1 Data Gas",
            &format!(
                "max_amount={:#x}, max_price={:#x}",
                rb.l1_data_gas.max_amount, rb.l1_data_gas.max_price_per_unit
            ),
        )
        .field(
            "Resource Bounds L2 Gas",
            &format!(
                "max_amount={:#x}, max_price={:#x}",
                rb.l2_gas.max_amount, rb.l2_gas.max_price_per_unit
            ),
        )
    }

    fn max_fee(self, fee: &Felt) -> Self {
        self.short_felt_field("Max Fee", fee)
    }

    fn tip(self, tip: u64) -> Self {
        self.field("Tip", &tip.to_string())
    }

    fn nonce_da_mode(self, mode: &DataAvailabilityMode) -> Self {
        self.field("Nonce DA Mode", fmt_da(mode))
    }

    fn fee_da_mode(self, mode: &DataAvailabilityMode) -> Self {
        self.field("Fee DA Mode", fmt_da(mode))
    }

    fn felt_field(self, label: &str, felt: &Felt) -> Self {
        self.field(label, &format!("{felt:#066x}"))
    }

    fn short_felt_field(self, label: &str, felt: &Felt) -> Self {
        self.field(label, &format!("{felt:#x}"))
    }

    fn felt_list_field(self, label: &str, felts: &[Felt]) -> Self {
        let formatted = if felts.is_empty() {
            "[]".to_string()
        } else {
            let inner: Vec<String> = felts.iter().map(|f| format!("{f:#x}")).collect();
            format!("[{}]", inner.join(", "))
        };
        self.field(label, &formatted)
    }
}

fn fmt_invoke_v0(
    tx: &starknet_rust::core::types::InvokeTransactionV0,
    tx_type: &str,
    tx_version: Option<&str>,
) -> String {
    let starknet_rust::core::types::InvokeTransactionV0 {
        transaction_hash,
        max_fee,
        signature,
        contract_address,
        entry_point_selector,
        calldata,
    } = tx;
    make_tx_output(tx_type, tx_version, transaction_hash)
        .contract_address(contract_address)
        .entry_point_selector(entry_point_selector)
        .calldata(calldata)
        .max_fee(max_fee)
        .signature(signature)
        .build()
}

fn fmt_invoke_v1(
    tx: &starknet_rust::core::types::InvokeTransactionV1,
    tx_type: &str,
    tx_version: Option<&str>,
) -> String {
    let starknet_rust::core::types::InvokeTransactionV1 {
        transaction_hash,
        sender_address,
        calldata,
        max_fee,
        signature,
        nonce,
    } = tx;
    make_tx_output(tx_type, tx_version, transaction_hash)
        .sender_address(sender_address)
        .transaction_nonce(nonce)
        .max_fee(max_fee)
        .calldata(calldata)
        .signature(signature)
        .build()
}

fn fmt_invoke_v3(
    tx: &starknet_rust::core::types::InvokeTransactionV3,
    tx_type: &str,
    tx_version: Option<&str>,
) -> String {
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
    } = tx;

    make_tx_output(tx_type, tx_version, transaction_hash)
        .sender_address(sender_address)
        .transaction_nonce(nonce)
        .calldata(calldata)
        .signature(signature)
        .resource_bounds(resource_bounds)
        .tip(*tip)
        .paymaster_data(paymaster_data)
        .nonce_da_mode(nonce_data_availability_mode)
        .fee_da_mode(fee_data_availability_mode)
        .account_deployment_data(account_deployment_data)
        .build()
}

fn fmt_declare_v0(
    tx: &starknet_rust::core::types::DeclareTransactionV0,
    tx_type: &str,
    tx_version: Option<&str>,
) -> String {
    let starknet_rust::core::types::DeclareTransactionV0 {
        transaction_hash,
        sender_address,
        max_fee,
        signature,
        class_hash,
    } = tx;
    make_tx_output(tx_type, tx_version, transaction_hash)
        .sender_address(sender_address)
        .class_hash(class_hash)
        .max_fee(max_fee)
        .signature(signature)
        .build()
}

fn fmt_declare_v1(
    tx: &starknet_rust::core::types::DeclareTransactionV1,
    tx_type: &str,
    tx_version: Option<&str>,
) -> String {
    let starknet_rust::core::types::DeclareTransactionV1 {
        transaction_hash,
        sender_address,
        max_fee,
        signature,
        nonce,
        class_hash,
    } = tx;
    make_tx_output(tx_type, tx_version, transaction_hash)
        .sender_address(sender_address)
        .transaction_nonce(nonce)
        .class_hash(class_hash)
        .max_fee(max_fee)
        .signature(signature)
        .build()
}

fn fmt_declare_v2(
    tx: &starknet_rust::core::types::DeclareTransactionV2,
    tx_type: &str,
    tx_version: Option<&str>,
) -> String {
    let starknet_rust::core::types::DeclareTransactionV2 {
        transaction_hash,
        sender_address,
        compiled_class_hash,
        max_fee,
        signature,
        nonce,
        class_hash,
    } = tx;
    make_tx_output(tx_type, tx_version, transaction_hash)
        .sender_address(sender_address)
        .transaction_nonce(nonce)
        .class_hash(class_hash)
        .compiled_class_hash(compiled_class_hash)
        .max_fee(max_fee)
        .signature(signature)
        .build()
}

fn fmt_declare_v3(
    tx: &starknet_rust::core::types::DeclareTransactionV3,
    tx_type: &str,
    tx_version: Option<&str>,
) -> String {
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

    make_tx_output(tx_type, tx_version, transaction_hash)
        .sender_address(sender_address)
        .transaction_nonce(nonce)
        .class_hash(class_hash)
        .compiled_class_hash(compiled_class_hash)
        .signature(signature)
        .resource_bounds(resource_bounds)
        .tip(*tip)
        .paymaster_data(paymaster_data)
        .nonce_da_mode(nonce_data_availability_mode)
        .fee_da_mode(fee_data_availability_mode)
        .account_deployment_data(account_deployment_data)
        .build()
}

fn fmt_deploy(
    tx: &starknet_rust::core::types::DeployTransaction,
    tx_type: &str,
    tx_version: Option<&str>,
) -> String {
    let starknet_rust::core::types::DeployTransaction {
        transaction_hash,
        version,
        contract_address_salt,
        constructor_calldata,
        class_hash,
    } = tx;
    make_tx_output(tx_type, tx_version, transaction_hash)
        .transaction_version(version)
        .class_hash(class_hash)
        .contract_address_salt(contract_address_salt)
        .constructor_calldata(constructor_calldata)
        .build()
}

fn fmt_deploy_account_v1(
    tx: &starknet_rust::core::types::DeployAccountTransactionV1,
    tx_type: &str,
    tx_version: Option<&str>,
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
    make_tx_output(tx_type, tx_version, transaction_hash)
        .transaction_nonce(nonce)
        .class_hash(class_hash)
        .contract_address_salt(contract_address_salt)
        .constructor_calldata(constructor_calldata)
        .max_fee(max_fee)
        .signature(signature)
        .build()
}

fn fmt_deploy_account_v3(
    tx: &starknet_rust::core::types::DeployAccountTransactionV3,
    tx_type: &str,
    tx_version: Option<&str>,
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

    make_tx_output(tx_type, tx_version, transaction_hash)
        .transaction_nonce(nonce)
        .class_hash(class_hash)
        .contract_address_salt(contract_address_salt)
        .constructor_calldata(constructor_calldata)
        .signature(signature)
        .resource_bounds(resource_bounds)
        .tip(*tip)
        .paymaster_data(paymaster_data)
        .nonce_da_mode(nonce_data_availability_mode)
        .fee_da_mode(fee_data_availability_mode)
        .build()
}

fn fmt_l1_handler(
    tx: &starknet_rust::core::types::L1HandlerTransaction,
    tx_type: &str,
    tx_version: Option<&str>,
) -> String {
    let starknet_rust::core::types::L1HandlerTransaction {
        transaction_hash,
        version,
        nonce,
        contract_address,
        entry_point_selector,
        calldata,
    } = tx;
    make_tx_output(tx_type, tx_version, transaction_hash)
        .transaction_version(version)
        .contract_address(contract_address)
        .entry_point_selector(entry_point_selector)
        .field("Nonce", &nonce.to_string())
        .calldata(calldata)
        .build()
}

fn make_tx_output(tx_type: &str, tx_version: Option<&str>, hash: &Felt) -> OutputBuilder {
    let b = OutputBuilder::new()
        .success_message("Transaction found")
        .blank_line()
        .field("Type", tx_type)
        .transaction_hash(hash);
    match tx_version {
        Some(v) => b.field("Version", v),
        None => b,
    }
}

fn fmt_da(mode: &DataAvailabilityMode) -> &'static str {
    match mode {
        DataAvailabilityMode::L1 => "L1",
        DataAvailabilityMode::L2 => "L2",
    }
}
