use std::cmp::PartialEq;
use std::collections::HashSet;

use cairo_lang_sierra::program::BranchTarget;
use cairo_lang_sierra::program::GenStatement;

use crate::config::GraphConfig;
use crate::decompiler::function::SierraStatement;

/// A struct representing a control flow graph (CFG) for a function
///
/// - Breaks down the function into basic blocks
/// - Captures control flow between blocks with edges
/// - Each block represents a sequence of statements executed sequentially
/// - Edges denote control flow transfers (conditional branches, jumps, fallthroughs)
#[derive(Debug, Clone)]
pub struct ControlFlowGraph {
    /// Function name or ID
    function_name: String,
    /// List of statements in the function
    statements: Vec<SierraStatement>,
    /// List of basic blocks in the CFG
    pub basic_blocks: Vec<BasicBlock>,
}

impl<'a> ControlFlowGraph {
    /// Creates a new `ControlFlowGraph` instance
    pub fn new(function_name: String, statements: Vec<SierraStatement>) -> Self {
        Self {
            function_name,
            statements,
            basic_blocks: Vec::new(),
        }
    }

    /// Gets the start and end offsets of basic blocks within the function's control flow graph
    pub fn get_basic_blocks_delimitations(&self) -> (Vec<u32>, Vec<u32>) {
        // Initialize vectors to store the start and end offsets of basic blocks
        let mut basic_blocks_starts = vec![];
        let mut basic_blocks_ends = vec![];

        // Iterate over each statement in the function
        for statement in &self.statements {
            // Match the type of statement
            match &statement.statement {
                // If it's a return statement, add its offset to the list of basic block ends
                GenStatement::Return(_) => {
                    basic_blocks_ends.push(statement.offset);
                }
                // If it's an invocation statement
                GenStatement::Invocation(invocation) => {
                    // Iterate over each branch target of the invocation
                    for target in &invocation.branches {
                        // Match the branch target type
                        match &target.target {
                            // If it's a statement target
                            BranchTarget::Statement(statement_idx) => {
                                // Add the offset of the statement after the invocation as the start of a new basic block
                                basic_blocks_starts.push(statement.offset + 1);
                                // Add the offset of the targeted statement as the start of a new basic block
                                basic_blocks_starts.push(statement_idx.0.try_into().unwrap());
                                // Add the offset of the targeted statement as the end of the current basic block
                                basic_blocks_ends.push(statement.offset);
                            }
                            // Ignore other types of branch targets
                            _ => {}
                        }
                    }
                }
            }
        }

        // Return the vectors containing the start and end offsets of basic blocks
        (basic_blocks_starts, basic_blocks_ends)
    }

    /// Generates the CFG basic blocks
    pub fn generate_basic_blocks(&mut self) {
        // Check if there are no statements and return early
        if self.statements.is_empty() {
            return;
        }

        // Retrieve basic blocks delimitations
        let (basic_blocks_starts, basic_blocks_ends) = self.get_basic_blocks_delimitations();

        // Initialize variables for tracking the current basic block
        let mut new_basic_block = true;
        let mut current_basic_block = BasicBlock::new(self.statements[0].clone());

        // Iterate through each statement
        for i in 0..self.statements.len() {
            let statement = &self.statements[i];

            // Check if the current statement marks the beginning of a new basic block
            if basic_blocks_starts.contains(&statement.offset) {
                // If it's the beginning of a new basic block, push the previous one to the list
                if new_basic_block {
                    self.basic_blocks.push(current_basic_block.clone());
                }
                // Create a new basic block
                current_basic_block = BasicBlock::new(statement.clone());
                new_basic_block = false;
            }

            // Add the current statement to the current basic block
            current_basic_block.statements.push(statement.clone());

            // Check if the current statement marks the end of the current basic block
            if basic_blocks_ends.contains(&statement.offset) {
                new_basic_block = true;
            }

            // Handle conditional branches
            if let Some(conditional_branch) = statement.as_conditional_branch(vec![]) {
                if let Some(edge_2_offset) = conditional_branch.edge_2_offset {
                    // Conditional branch with 2 edges (JNZ)
                    current_basic_block.edges.push(Edge {
                        source: statement.offset,
                        destination: conditional_branch.edge_1_offset.unwrap(),
                        edge_type: EdgeType::ConditionalTrue,
                    });
                    current_basic_block.edges.push(Edge {
                        source: statement.offset,
                        destination: edge_2_offset + 1,
                        edge_type: EdgeType::ConditionalFalse,
                    });
                } else if let Some(edge_1_offset) = conditional_branch.edge_1_offset {
                    // Conditional jump with 1 edge (JUMP)
                    current_basic_block.edges.push(Edge {
                        source: statement.offset,
                        destination: edge_1_offset,
                        edge_type: EdgeType::Unconditional,
                    });
                }
            }
            // Check for fallthrough edges
            else if i < (self.statements.len() - 1) {
                if basic_blocks_starts.contains(&(self.statements[i + 1].offset))
                    && !matches!(statement.statement, GenStatement::Return(_))
                {
                    // Fallthrough edge
                    current_basic_block.edges.push(Edge {
                        source: statement.offset,
                        destination: statement.offset + 1,
                        edge_type: EdgeType::Fallthrough,
                    });
                }
            }
        }

        // Push the last basic block to the list
        self.basic_blocks.push(current_basic_block);
    }

    /// Returns all the possible paths in a function
    pub fn paths(&self) -> Vec<Vec<&BasicBlock>> {
        let mut paths = Vec::new();

        // Find the starting blocks
        let mut start_blocks = Vec::new();
        for block in &self.basic_blocks {
            if self.parents(block).is_empty() {
                start_blocks.push(block);
            }
        }

        // Perform DFS from each start block to find all paths
        for start_block in start_blocks {
            let mut stack = vec![(vec![start_block], start_block)];
            while let Some((current_path, current_block)) = stack.pop() {
                let children = self.children(current_block);
                if children.is_empty() {
                    paths.push(current_path.clone());
                } else {
                    for child in children {
                        let mut new_path = current_path.clone();
                        new_path.push(child);
                        stack.push((new_path, child));
                    }
                }
            }
        }

        paths
    }

    /// Returns the children blocks of a basic block
    fn children(&self, block: &BasicBlock) -> Vec<&BasicBlock> {
        let mut children = Vec::new();
        let edges_destinations: HashSet<_> =
            block.edges.iter().map(|edge| edge.destination).collect();

        for basic_block in &self.basic_blocks {
            if edges_destinations.contains(&basic_block.start_offset) {
                children.push(basic_block);
            }
        }
        children
    }

    /// Returns the parent blocks of a basic block
    fn parents(&self, block: &BasicBlock) -> Vec<&BasicBlock> {
        let mut parents = Vec::new();
        let start_offset = block.start_offset;

        for basic_block in &self.basic_blocks {
            let edges_offset: Vec<_> = basic_block
                .edges
                .iter()
                .map(|edge| edge.destination)
                .collect();
            if edges_offset.contains(&start_offset) {
                parents.push(basic_block);
            }
        }
        parents
    }

    /// Generates the DOT format subgraph for function CFG
    pub fn generate_dot_graph(&self) -> String {
        let mut dot_graph = format!("\tsubgraph \"cluster_{}\" {{\n", self.function_name);
        dot_graph += &format!("\t\tlabel=\"{}\"\n", self.function_name);
        dot_graph += &format!(
            "\t\tfontname=\"{}\";\n",
            GraphConfig::CFG_GRAPH_ATTR_FONTNAME
        );
        dot_graph += &format!("\t\tfontsize={};\n", GraphConfig::CFG_GRAPH_ATTR_FONTSIZE);

        // Iterate over each basic block to create nodes
        for block in &self.basic_blocks {
            let mut label_instruction = String::new();
            for statement in &block.statements {
                label_instruction += &format!(
                    "{} : {}\t\t\\l",
                    statement.offset,
                    statement.raw_statement()
                );
            }

            dot_graph += &format!(
                "\t\t\"{}\" [label=\"{}\" shape=\"box\" style=\"{}\" fillcolor=\"{}\" color=\"{}\" fontname=\"{}\" margin=\"{}\"];\n",
                block.name,
                label_instruction,
                GraphConfig::CFG_NODE_ATTR_STYLE,
                GraphConfig::CFG_NODE_ATTR_FILLCOLOR,
                GraphConfig::CFG_NODE_ATTR_COLOR,
                GraphConfig::CFG_NODE_ATTR_FONTNAME,
                GraphConfig::CFG_NODE_ATTR_MARGIN
            );
        }

        // Add edges between nodes
        for block in &self.basic_blocks {
            for edge in &block.edges {
                let color = match edge.edge_type {
                    EdgeType::ConditionalTrue => GraphConfig::EDGE_CONDITIONAL_TRUE_COLOR,
                    EdgeType::ConditionalFalse => GraphConfig::EDGE_CONDITIONAL_FALSE_COLOR,
                    EdgeType::Unconditional => GraphConfig::EDGE_UNCONDITIONAL_COLOR,
                    EdgeType::Fallthrough => GraphConfig::EDGE_FALLTHROUGH_COLOR,
                };
                if self
                    .basic_blocks
                    .iter()
                    .any(|b| b.start_offset == edge.destination)
                {
                    dot_graph += &format!(
                        "\t\t\"{}\" -> \"{}\" [color=\"{}\" arrowsize={} fontname=\"{}\" labeldistance={} labelfontcolor=\"{}\" penwidth={}];\n",
                        block.name,
                        self.get_block_name_by_offset(edge.destination),
                        color,
                        GraphConfig::CFG_EDGE_ATTR_ARROWSIZE,
                        GraphConfig::CFG_EDGE_ATTR_FONTNAME,
                        GraphConfig::CFG_EDGE_ATTR_LABELDISTANCE,
                        GraphConfig::CFG_EDGE_ATTR_LABELFONTCOLOR,
                        GraphConfig::CFG_EDGE_ATTR_PENWIDTH
                    );
                }
            }
        }

        dot_graph += "\t}\n";
        dot_graph
    }

    /// Retrieves the name of a basic block based on its start offset
    fn get_block_name_by_offset(&self, offset: u32) -> String {
        self.basic_blocks
            .iter()
            .find(|&b| b.start_offset == offset)
            .map_or(String::from("Unknown"), |b| b.name.clone())
    }
}

/// Enum representing different types of CFG edges
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeType {
    Unconditional,
    ConditionalTrue,
    ConditionalFalse,
    Fallthrough,
}

/// Struct representing a control flow graph (CFG) edge
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Edge {
    source: u32,
    pub destination: u32,
    pub edge_type: EdgeType,
}

impl Edge {
    /// Creates a new `Edge` instance
    #[allow(dead_code)]
    pub fn new(source: u32, destination: u32, edge_type: EdgeType) -> Self {
        Self {
            source,
            destination,
            edge_type,
        }
    }
}

/// Struct representing a Sierra Control-Flow Graph basic block
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub start_offset: u32,
    end_offset: Option<u32>,
    /// Name of the basic block
    name: String,
    /// Instructions (statements) in the basic block
    pub statements: Vec<SierraStatement>,
    /// Edges of the basic block
    pub edges: Vec<Edge>,
}

impl BasicBlock {
    /// Creates a new `BasicBlock` instance
    pub fn new(start_statement: SierraStatement) -> Self {
        let start_offset = start_statement.offset;
        // Name the basicblock using the start offset
        let name = format!("bb_{}", start_offset);

        BasicBlock {
            start_offset,
            end_offset: None,
            name,
            statements: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Returns the name of the basic block
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Adds a statement to the basic block
    #[inline]
    pub fn add_statement(&mut self, statement: SierraStatement) {
        self.statements.push(statement);
    }

    /// Adds an edge to the basic block
    #[inline]
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Sets the end offset of the basic block
    #[inline]
    pub fn set_end_offset(&mut self, end_offset: u32) {
        self.end_offset = Some(end_offset);
    }
}

impl PartialEq for BasicBlock {
    fn eq(&self, other: &Self) -> bool {
        // Compare based on the start_offset
        self.start_offset == other.start_offset
    }
}

/// Struct representing a Sierra conditional branch
#[allow(dead_code)]
#[derive(Debug)]
pub struct SierraConditionalBranch {
    // Inherit SierraStatement's fields
    statement: SierraStatement,
    // Function name
    pub function: String,
    // TODO: Create a Variable object
    pub parameters: Vec<String>,
    // Edges offsets
    pub edge_1_offset: Option<u32>,
    pub edge_2_offset: Option<u32>,
    // Fallthrough conditional branch
    fallthrough: bool,
}

impl SierraConditionalBranch {
    /// Creates a new `SierraConditionalBranch` instance
    pub fn new(
        statement: SierraStatement,
        function: String,
        parameters: Vec<String>,
        edge_1_offset: Option<u32>,
        edge_2_offset: Option<u32>,
        fallthrough: bool,
    ) -> Self {
        let mut edge_2_offset = edge_2_offset;
        if fallthrough && edge_2_offset.is_none() {
            edge_2_offset = Some(statement.offset);
        }

        SierraConditionalBranch {
            statement,
            function,
            parameters,
            edge_1_offset,
            edge_2_offset,
            fallthrough,
        }
    }
}
