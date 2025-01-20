pub struct GraphConfig;

#[allow(dead_code)]
impl GraphConfig {
    // Node attributes for CFG
    pub const CFG_NODE_ATTR_STYLE: &'static str = "filled, solid";
    pub const CFG_NODE_ATTR_SHAPE: &'static str = "rect, plaintext";
    pub const CFG_NODE_ATTR_COLOR: &'static str = "#9E9E9E";
    pub const CFG_NODE_ATTR_FILLCOLOR: &'static str = "#F5F5F5";
    pub const CFG_NODE_ATTR_FONTNAME: &'static str = "Helvetica,Arial,sans-serif";
    pub const CFG_NODE_ATTR_MARGIN: &'static str = "0.2";

    // Graph attributes for CFG
    pub const CFG_GRAPH_ATTR_OVERLAP: &'static str = "scale";
    pub const CFG_GRAPH_ATTR_FONTNAME: &'static str = "Helvetica,Arial,sans-serif";
    pub const CFG_GRAPH_ATTR_FONTSIZE: &'static str = "20";
    pub const CFG_GRAPH_ATTR_LAYOUT: &'static str = "dot";
    pub const CFG_GRAPH_ATTR_NEWRANK: &'static str = "true";

    // Edge attributes for CFG
    pub const CFG_EDGE_ATTR_ARROWSIZE: &'static str = "0.5";
    pub const CFG_EDGE_ATTR_FONTNAME: &'static str = "Helvetica,Arial,sans-serif";
    pub const CFG_EDGE_ATTR_LABELDISTANCE: &'static str = "3";
    pub const CFG_EDGE_ATTR_LABELFONTCOLOR: &'static str = "#00000080";
    pub const CFG_EDGE_ATTR_PENWIDTH: &'static str = "2";

    // Edge colors
    pub const EDGE_CONDITIONAL_TRUE_COLOR: &'static str = "#8BC34A";
    pub const EDGE_CONDITIONAL_FALSE_COLOR: &'static str = "#C62828";
    pub const EDGE_UNCONDITIONAL_COLOR: &'static str = "#0D47A1";
    pub const EDGE_FALLTHROUGH_COLOR: &'static str = "#212121";

    // Node attributes for callgraph
    pub const CALLGRAPH_NODE_ATTR_STYLE: &'static str = "filled";
    pub const CALLGRAPH_NODE_ATTR_SHAPE: &'static str = "rect, plaintext";
    pub const CALLGRAPH_NODE_ATTR_PENCOLOR: &'static str = "#00000044";
    pub const CALLGRAPH_NODE_ATTR_FONTNAME: &'static str = "Helvetica,Arial,sans-serif";

    // Grapĥ attributes for callgraph
    pub const CALLGRAPH_GRAPH_ATTR_FONTNAME: &'static str = "Helvetica,Arial,sans-serif";
    pub const CALLGRAPH_GRAPH_ATTR_FONTSIZE: &'static str = "20";
    pub const CALLGRAPH_GRAPH_ATTR_LAYOUT: &'static str = "dot";
    pub const CALLGRAPH_GRAPH_ATTR_RANKDIR: &'static str = "LR";
    pub const CALLGRAPH_GRAPH_ATTR_NEWRANK: &'static str = "true";

    // Edge attributes for callgraph
    pub const CALLGRAPH_EDGE_ATTR_ARROWSIZE: &'static str = "0.5";
    pub const CALLGRAPH_EDGE_ATTR_FONTNAME: &'static str = "Helvetica,Arial,sans-serif";
    pub const CALLGRAPH_EDGE_ATTR_LABELDISTANCE: &'static str = "3";
    pub const CALLGRAPH_EDGE_ATTR_LABELFONTCOLOR: &'static str = "#00000080";
    pub const CALLGRAPH_EDGE_ATTR_PENWIDTH: &'static str = "2";

    // Callgraph colors
    pub const CALLGRAPH_USER_DEFINED_FUNCTIONS_COLOR: &'static str = "#95D2B3";
    pub const CALLGRAPH_LIBFUNCS_COLOR: &'static str = "#E86356";
}
