use crate::trace::types::{ContractTrace, Trace, TraceInfo};
use crate::tree::building::node::Node;

/// Trait for adding a type to a tree.
/// Implementations of this trait should only focus on placements of nodes in a tree not display aspects of them.
/// Display should be handled by the [`NodeDisplay`](super::display::NodeDisplay) trait.
pub trait AsTreeNode {
    fn as_tree_node(&self, parent: &mut Node);
}

impl AsTreeNode for Trace {
    fn as_tree_node(&self, parent: &mut Node) {
        let mut node = parent.child_node(&self.test_name);
        for nested_call in &self.nested_calls {
            node.as_tree_node(nested_call);
        }
    }
}

impl AsTreeNode for ContractTrace {
    fn as_tree_node(&self, parent: &mut Node) {
        parent
            .child_node(&self.selector)
            .as_tree_node(&self.trace_info);
    }
}

impl AsTreeNode for TraceInfo {
    fn as_tree_node(&self, parent: &mut Node) {
        parent.leaf(&self.contract_name);
        parent.leaf_optional(self.entry_point_type.as_option());
        parent.leaf_optional(self.calldata.as_option());
        parent.leaf_optional(self.storage_address.as_option());
        parent.leaf_optional(self.caller_address.as_option());
        parent.leaf_optional(self.call_type.as_option());
        parent.leaf_optional(self.call_result.as_option());
        for nested_call in &self.nested_calls {
            parent.as_tree_node(nested_call);
        }
    }
}
