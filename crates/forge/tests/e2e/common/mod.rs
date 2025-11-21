#[cfg(not(feature = "cairo-native"))]
use cairo_annotations::trace_data::{
    CallTraceNode as ProfilerCallTraceNode, CallTraceV1 as ProfilerCallTrace,
};

mod output;
pub mod runner;

#[cfg(not(feature = "cairo-native"))]
pub fn get_trace_from_trace_node(trace_node: &ProfilerCallTraceNode) -> &ProfilerCallTrace {
    if let ProfilerCallTraceNode::EntryPointCall(trace) = trace_node {
        trace
    } else {
        panic!("Deploy without constructor node was not expected")
    }
}
