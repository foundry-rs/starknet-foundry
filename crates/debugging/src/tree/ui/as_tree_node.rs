use crate::trace::{Trace, TraceInfo};
use crate::tree::building::node::Node;

/// Trait for adding a type to a tree.
/// Implementations of this trait should only focus on placements of nodes in a tree not display aspects of them.
/// Display should be handled by the [`NodeDisplay`](super::display::NodeDisplay) trait.
pub trait AsTreeNode {
    fn as_tree_node(&self, parent: &mut Node);
}

impl AsTreeNode for Trace {
    fn as_tree_node(&self, parent: &mut Node) {
        parent
            .child_node(&self.selector)
            .as_tree_node(&self.trace_info);
    }
}

impl AsTreeNode for TraceInfo {
    fn as_tree_node(&self, parent: &mut Node) {
        parent.leaf(&self.entry_point_type);
        parent.leaf(&self.calldata);
        parent.leaf(&self.storage_address);
        parent.leaf(&self.caller_address);
        parent.leaf(&self.call_type);
        parent.leaf(&self.call_result);
        for nested_call in &self.nested_calls {
            parent.as_tree_node(nested_call);
        }
    }
}
