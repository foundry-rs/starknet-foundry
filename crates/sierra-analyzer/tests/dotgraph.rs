use sierra_analyzer_lib::sierra_program::SierraProgram;

#[test]
fn test_dogtgraph_cfg_output() {
    // Read file content
    let content = include_str!("../examples/sierra/fib.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);

    // Decompile the sierra program with a colorless output
    let use_color = false;
    decompiler.decompile(use_color);

    // Generate CFG dotgraph
    let cfg_dotgraph = decompiler.generate_cfg();

    // Expected dotgraph
    let expected_output = "digraph {\n\tgraph [fontname=\"Helvetica,Arial,sans-serif\" fontsize=20 layout=dot newrank=true overlap=scale];\n\tnode [color=\"#9E9E9E\" fillcolor=\"#F5F5F5\" fontname=\"Helvetica,Arial,sans-serif\" margin=0.2 shape=\"rect, plaintext\" style=\"filled, solid\"];\n\tedge [arrowsize=0.5 fontname=\"Helvetica,Arial,sans-serif\" labeldistance=3 labelfontcolor=\"#00000080\" penwidth=2];\n\tsubgraph \"cluster_examples::fib::fib\" {\n\t\tlabel=\"examples::fib::fib\"\n\t\tfontname=\"Helvetica,Arial,sans-serif\";\n\t\tfontsize=20;\n\t\t\"bb_0\" [label=\"0 : disable_ap_tracking() -> ()\t\t\\l1 : dup<felt252>([2]) -> ([2], [3])\t\t\\l2 : felt252_is_zero([3]) { fallthrough() 8([4]) }\t\t\\l\" shape=\"box\" style=\"filled, solid\" fillcolor=\"#F5F5F5\" color=\"#9E9E9E\" fontname=\"Helvetica,Arial,sans-serif\" margin=\"0.2\"];\n\t\t\"bb_3\" [label=\"3 : branch_align() -> ()\t\t\\l4 : drop<felt252>([1]) -> ()\t\t\\l5 : drop<felt252>([2]) -> ()\t\t\\l6 : store_temp<felt252>([0]) -> ([0])\t\t\\l7 : return([0])\t\t\\l\" shape=\"box\" style=\"filled, solid\" fillcolor=\"#F5F5F5\" color=\"#9E9E9E\" fontname=\"Helvetica,Arial,sans-serif\" margin=\"0.2\"];\n\t\t\"bb_8\" [label=\"8 : branch_align() -> ()\t\t\\l9 : drop<NonZero<felt252>>([4]) -> ()\t\t\\l10 : dup<felt252>([1]) -> ([1], [5])\t\t\\l11 : felt252_add([0], [5]) -> ([6])\t\t\\l12 : const_as_immediate<Const<felt252, 1>>() -> ([7])\t\t\\l13 : felt252_sub([2], [7]) -> ([8])\t\t\\l14 : store_temp<felt252>([1]) -> ([1])\t\t\\l15 : store_temp<felt252>([6]) -> ([6])\t\t\\l16 : store_temp<felt252>([8]) -> ([8])\t\t\\l17 : function_call<user@examples::fib::fib>([1], [6], [8]) -> ([9])\t\t\\l18 : return([9])\t\t\\l\" shape=\"box\" style=\"filled, solid\" fillcolor=\"#F5F5F5\" color=\"#9E9E9E\" fontname=\"Helvetica,Arial,sans-serif\" margin=\"0.2\"];\n\t\t\"bb_0\" -> \"bb_8\" [color=\"#8BC34A\" arrowsize=0.5 fontname=\"Helvetica,Arial,sans-serif\" labeldistance=3 labelfontcolor=\"#00000080\" penwidth=2];\n\t\t\"bb_0\" -> \"bb_3\" [color=\"#C62828\" arrowsize=0.5 fontname=\"Helvetica,Arial,sans-serif\" labeldistance=3 labelfontcolor=\"#00000080\" penwidth=2];\n\t}\n}\n";

    assert_eq!(cfg_dotgraph, expected_output);
}

#[test]
fn test_dogtgraph_callgraph_output() {
    // Read file content
    let content = include_str!("../examples/sierra/fib.sierra").to_string();

    // Init a new SierraProgram with the .sierra file content
    let program = SierraProgram::new(content);

    // Don't use the verbose output
    let verbose_output = false;

    // Decompile the Sierra program
    let mut decompiler = program.decompiler(verbose_output);

    // Decompile the sierra program with a colorless output
    let use_color = false;
    decompiler.decompile(use_color);

    // Generate Callgraph dotgraph
    let callgraph_dotgraph = decompiler.generate_callgraph();

    // Expected dotgraph
    let expected_output = "strict digraph G {\n    graph [fontname=\"Helvetica,Arial,sans-serif\", fontsize=20, layout=\"dot\", rankdir=\"LR\", newrank=true];\n    node [style=\"filled\", shape=\"rect, plaintext\", pencolor=\"#00000044\", margin=\"0.5,0.1\", fontname=\"Helvetica,Arial,sans-serif\"];\n    edge [arrowsize=0.5, fontname=\"Helvetica,Arial,sans-serif\", labeldistance=3, labelfontcolor=\"#00000080\", penwidth=2];\n   \"examples::fib::fib\" [shape=\"rectangle, fill\", fillcolor=\"#95D2B3\", style=\"filled\"];\n   \"const_as_immediate<Const<felt252, 1>>\t\t\" [shape=\"rectangle\", fillcolor=\"#E86356\", style=\"filled\"];\n   \"examples::fib::fib\" -> \"const_as_immediate<Const<felt252, 1>>\t\t\";\n   \"examples::fib::fib\" [shape=\"rectangle\", fillcolor=\"#95D2B3\", style=\"filled\"];\n   \"examples::fib::fib\" -> \"examples::fib::fib\";\n}\n";

    assert_eq!(callgraph_dotgraph, expected_output);
}
