use graphviz_rust::cmd::Format;
use graphviz_rust::dot_structures::*;
use graphviz_rust::exec;
use graphviz_rust::parse;
use graphviz_rust::printer::PrinterContext;
use std::fs::File;
use std::io::{self, Write};

/// Converts a DOT graph provided as a string to SVG format and saves it to a file
pub fn save_svg_graph_to_file(filename: &str, graph: String) -> io::Result<()> {
    // Parse the graph from the string input
    let parsed_graph: Graph = parse(&graph).unwrap();

    // Generate SVG output
    let svg_data = exec(
        parsed_graph,
        &mut PrinterContext::default(),
        vec![Format::Svg.into()],
    )?;

    // Create the output file and write the SVG data to it
    let mut file = File::create(filename)?;
    file.write_all(&svg_data)?;

    Ok(())
}
