use crate::trace::types::{
    CallerAddress, ContractAddress, ContractName, Selector, TestName, TransformedCallResult,
    TransformedCalldata,
};
use blockifier::execution::entry_point::CallType;
use starknet_api::contract_class::EntryPointType;
use starknet_types_core::felt::Felt;
use std::fmt::Debug;

/// Trait controlling the display of a node in a tree.
/// All nodes should have a tag that explains what the node represents
/// and a pretty string representation of data held by the node.
pub trait NodeDisplay {
    const TAG: &'static str;
    fn string_pretty(&self) -> String;

    fn display(&self) -> String {
        let tag = console::style(Self::TAG).magenta();
        let content = self.string_pretty();
        format!("[{tag}] {content}")
    }
}

impl NodeDisplay for TestName {
    const TAG: &'static str = "test name";
    fn string_pretty(&self) -> String {
        self.0.clone()
    }
}

impl NodeDisplay for ContractName {
    const TAG: &'static str = "contract name";
    fn string_pretty(&self) -> String {
        self.0.to_string()
    }
}

impl NodeDisplay for Selector {
    const TAG: &'static str = "selector";
    fn string_pretty(&self) -> String {
        self.0.to_string()
    }
}

impl NodeDisplay for EntryPointType {
    const TAG: &'static str = "entry point type";
    fn string_pretty(&self) -> String {
        string_debug(self)
    }
}

impl NodeDisplay for TransformedCalldata {
    const TAG: &'static str = "calldata";
    fn string_pretty(&self) -> String {
        self.0.clone()
    }
}

impl NodeDisplay for ContractAddress {
    const TAG: &'static str = "contract address";
    fn string_pretty(&self) -> String {
        string_hex(self.0)
    }
}

impl NodeDisplay for CallerAddress {
    const TAG: &'static str = "caller address";
    fn string_pretty(&self) -> String {
        string_hex(self.0)
    }
}

impl NodeDisplay for CallType {
    const TAG: &'static str = "call type";
    fn string_pretty(&self) -> String {
        string_debug(self)
    }
}

impl NodeDisplay for TransformedCallResult {
    const TAG: &'static str = "call result";
    fn string_pretty(&self) -> String {
        self.0.clone()
    }
}

/// Helper function to get hex representation
/// of a type that can be converted to a [`Felt`].
fn string_hex(data: impl Into<Felt>) -> String {
    data.into().to_hex_string()
}

/// Helper function to get debug representation of a type as a string.
/// Mainly used for enums that hold no data or vectors of felts.
fn string_debug(data: impl Debug) -> String {
    format!("{data:?}")
}
