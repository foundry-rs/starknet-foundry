use cairo_annotations::annotations::coverage::{CodeLocation, ColumnNumber, LineNumber};
use cairo_annotations::annotations::profiler::FunctionName;
use starknet_api::core::ClassHash;
use std::fmt;
use std::fmt::Display;

pub struct Backtrace<'a> {
    pub code_location: &'a CodeLocation,
    pub function_name: &'a FunctionName,
    pub inlined: bool,
}

pub struct BacktraceStack<'a> {
    pub contract_name: &'a str,
    pub stack: Vec<Backtrace<'a>>,
}

impl Display for Backtrace<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let function_name = &self.function_name.0;
        let path = &self.code_location.0;
        let line = self.code_location.1.start.line + LineNumber(1); // most editors start line numbers from 1
        let col = self.code_location.1.start.col + ColumnNumber(1); // most editors start column numbers from 1

        if self.inlined {
            write!(f, "(inlined) ")?;
        }

        write!(f, "{function_name}\n       at {path}:{line}:{col}")
    }
}

impl Display for BacktraceStack<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "error occurred in contract '{}'", self.contract_name)?;
        writeln!(f, "stack backtrace:")?;
        for (i, backtrace) in self.stack.iter().enumerate() {
            writeln!(f, "   {i}: {backtrace}")?;
        }
        Ok(())
    }
}

pub fn render_fork_backtrace(contract_class_hash: &ClassHash) -> String {
    format!(
        "error occurred in forked contract with class hash: {:#x}\n",
        contract_class_hash.0
    )
}
