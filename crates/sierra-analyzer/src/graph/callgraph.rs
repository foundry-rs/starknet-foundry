use cairo_lang_sierra::program::GenStatement;

use crate::config::GraphConfig;
use crate::decompiler::function::Function;
use crate::decompiler::libfuncs_patterns::{
    IRRELEVANT_CALLGRAPH_FUNCTIONS_REGEXES, USER_DEFINED_FUNCTION_REGEX,
};
use crate::parse_element_name;

/// Generates the callgraph dotgraph from a vector of Function objects
pub fn process_callgraph(functions: &[Function]) -> String {
    let mut dot = String::from("strict digraph G {\n");

    // Global Graph configuration
    dot.push_str(&generate_graph_config());

    // Node attributes
    dot.push_str(&generate_node_attributes());

    // Edge attributes
    dot.push_str(&generate_edge_attributes());

    for function in functions {
        let function_name = format!("{}", parse_element_name!(function.function.id));

        // Constructing the node entry for DOT format
        dot.push_str(&generate_function_node(&function_name));

        for statement in &function.statements {
            if let GenStatement::Invocation(statement) = &statement.statement {
                let called_function = parse_element_name!(&statement.libfunc_id);

                // Add user-defined function to the callgraph
                if let Some(captures) = USER_DEFINED_FUNCTION_REGEX.captures(&called_function) {
                    if let Some(matched_group) = captures.name("function_id") {
                        let called_function_name = format!("{}", matched_group.as_str());
                        dot.push_str(&generate_user_defined_function_node(&called_function_name));
                        dot.push_str(&generate_edge(&function_name, &called_function_name));
                    }
                }
                // Add libfuncs to the callgraph
                else {
                    let called_function_name = format!("{}\t\t", called_function.as_str());

                    // Skip irrelevant functions
                    if IRRELEVANT_CALLGRAPH_FUNCTIONS_REGEXES
                        .iter()
                        .any(|regex| regex.is_match(&called_function_name))
                    {
                        continue;
                    }

                    dot.push_str(&generate_libfunc_node(&called_function_name));
                    dot.push_str(&generate_edge(&function_name, &called_function_name));
                }
            }
        }
    }

    dot.push_str("}\n");
    dot
}

/// Generates the graph configuration for the DOT format
fn generate_graph_config() -> String {
    format!(
        "    graph [fontname=\"{}\", fontsize={}, layout=\"{}\", rankdir=\"{}\", newrank={}];\n",
        GraphConfig::CALLGRAPH_GRAPH_ATTR_FONTNAME,
        GraphConfig::CALLGRAPH_GRAPH_ATTR_FONTSIZE,
        GraphConfig::CALLGRAPH_GRAPH_ATTR_LAYOUT,
        GraphConfig::CALLGRAPH_GRAPH_ATTR_RANKDIR,
        GraphConfig::CALLGRAPH_GRAPH_ATTR_NEWRANK,
    )
}

/// Generates the node attributes for the DOT format
fn generate_node_attributes() -> String {
    format!(
        "    node [style=\"{}\", shape=\"{}\", pencolor=\"{}\", margin=\"0.5,0.1\", fontname=\"{}\"];\n",
        GraphConfig::CALLGRAPH_NODE_ATTR_STYLE,
        GraphConfig::CALLGRAPH_NODE_ATTR_SHAPE,
        GraphConfig::CALLGRAPH_NODE_ATTR_PENCOLOR,
        GraphConfig::CALLGRAPH_NODE_ATTR_FONTNAME,
    )
}

/// Generates the edge attributes for the DOT format
fn generate_edge_attributes() -> String {
    format!(
        "    edge [arrowsize={}, fontname=\"{}\", labeldistance={}, labelfontcolor=\"{}\", penwidth={}];\n",
        GraphConfig::CALLGRAPH_EDGE_ATTR_ARROWSIZE,
        GraphConfig::CALLGRAPH_EDGE_ATTR_FONTNAME,
        GraphConfig::CALLGRAPH_EDGE_ATTR_LABELDISTANCE,
        GraphConfig::CALLGRAPH_EDGE_ATTR_LABELFONTCOLOR,
        GraphConfig::CALLGRAPH_EDGE_ATTR_PENWIDTH,
    )
}

/// Generates a function node for the DOT format
fn generate_function_node(function_name: &str) -> String {
    format!(
        "   \"{}\" [shape=\"rectangle, fill\", fillcolor=\"{}\", style=\"filled\"];\n",
        function_name,
        GraphConfig::CALLGRAPH_USER_DEFINED_FUNCTIONS_COLOR,
    )
}

/// Generates a user-defined function node for the DOT format
fn generate_user_defined_function_node(function_name: &str) -> String {
    format!(
        "   \"{}\" [shape=\"rectangle\", fillcolor=\"{}\", style=\"filled\"];\n",
        function_name,
        GraphConfig::CALLGRAPH_USER_DEFINED_FUNCTIONS_COLOR
    )
}

/// Generates a libfunc node for the DOT format
fn generate_libfunc_node(function_name: &str) -> String {
    format!(
        "   \"{}\" [shape=\"rectangle\", fillcolor=\"{}\", style=\"filled\"];\n",
        function_name,
        GraphConfig::CALLGRAPH_LIBFUNCS_COLOR
    )
}

/// Generates an edge for the DOT format
fn generate_edge(from: &str, to: &str) -> String {
    format!("   \"{}\" -> \"{}\";\n", from, to)
}
