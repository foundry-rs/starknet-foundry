#[cfg(not(feature = "run-test-native"))]
use cairo_annotations::trace_data::*;
pub mod runner;

#[cfg(not(feature = "run-test-native"))]
pub fn get_trace_from_trace_node(trace_node: &CallTraceNode) -> &CallTraceV1 {
    if let CallTraceNode::EntryPointCall(trace) = trace_node {
        trace
    } else {
        panic!("Deploy without constructor node was not expected")
    }
}
