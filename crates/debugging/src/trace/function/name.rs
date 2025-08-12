use cairo_lang_sierra::program::{Program, StatementIdx};
use regex::Regex;
use std::sync::LazyLock;

/// Represents a function name in the Sierra program.
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum FunctionName {
    NonInlined(String),
    // TODO: Add inlined variant in next PR.
}

impl FunctionName {
    /// Creates a [`FunctionName`] from a [`StatementIdx`] index and a [`Program`].
    pub fn from_program(statement_idx: StatementIdx, program: &Program) -> Self {
        let function_idx = program
            .funcs
            .partition_point(|f| f.entry_point.0 <= statement_idx.0)
            - 1;
        let function_name = program.funcs[function_idx].id.to_string();
        let function_name = remove_loop_suffix(&function_name);
        let function_name = remove_monomorphization_suffix(&function_name);
        FunctionName::NonInlined(function_name)
    }

    /// Returns the function name as a [`&str`].
    pub fn function_name(&self) -> &str {
        match self {
            FunctionName::NonInlined(name) => name,
        }
    }
}

/// Remove suffix in case of loop function e.g. `[expr36]`.
fn remove_loop_suffix(function_name: &str) -> String {
    static RE_LOOP_FUNC: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"\[expr\d*]")
            .expect("Failed to create regex for normalizing loop function names")
    });
    RE_LOOP_FUNC.replace(function_name, "").to_string()
}

/// Remove parameters from monomorphised Cairo generics e.g. `<felt252>`.
fn remove_monomorphization_suffix(function_name: &str) -> String {
    static RE_MONOMORPHIZATION: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"::<.*>")
            .expect("Failed to create regex for normalizing monomorphized generic function names")
    });
    RE_MONOMORPHIZATION.replace(function_name, "").to_string()
}
