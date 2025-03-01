use crate::trace::{Trace, TraceInfo};
use crate::tree::node::Node;

/// Trait for serializing a type into a tree.
/// Implementations of this trait should only focus on placements of nodes in a tree not display aspects of them.
/// Display should be handled by the [`NodeDisplay`](super::display::NodeDisplay) trait.
pub trait TreeSerialize {
    fn serialize(&self, node: &mut Node);
}

impl TreeSerialize for Trace {
    fn serialize(&self, node: &mut Node) {
        node.child_node(&self.selector).serialize(&self.trace_info);
    }
}

impl TreeSerialize for TraceInfo {
    fn serialize(&self, node: &mut Node) {
        node.leaf(&self.entry_point_type);
        node.leaf(&self.calldata);
        node.leaf(&self.storage_address);
        node.leaf(&self.caller_address);
        node.leaf(&self.call_type);
        node.leaf(&self.call_result);
        for nested_call in &self.nested_calls {
            node.serialize(nested_call);
        }
    }
}
