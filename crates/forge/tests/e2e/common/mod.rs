use trace_data::{CallTrace as ProfilerCallTrace, CallTraceNode as ProfilerCallTraceNode};

pub mod runner;

pub fn get_trace_from_trace_node(trace_node: &ProfilerCallTraceNode) -> &ProfilerCallTrace {
    if let ProfilerCallTraceNode::EntryPointCall(trace) = trace_node {
        trace
    } else {
        panic!("Deploy without constructor node was not expected")
    }
}
