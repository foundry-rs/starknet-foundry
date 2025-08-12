use crate::trace::function::name::FunctionName;

/// Represents a call stack for function calls, allowing to track the current function call
pub struct CallStack {
    stack: Vec<FunctionName>,
    previous_stack_lengths: Vec<usize>,
}

impl CallStack {
    /// Creates a new empty [`CallStack`]
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            previous_stack_lengths: Vec::new(),
        }
    }

    /// Enters a new function call by updating the stack with the new call stack.
    /// It saves the current stack length to allow returning to it later.
    ///
    /// The New call stack is expected to be a prefix of the current stack.
    pub fn enter_function_call(&mut self, new_call_stack: Vec<FunctionName>) {
        self.previous_stack_lengths.push(self.stack.len());

        self.stack = new_call_stack;
    }

    /// Exits the current function call by truncating the stack to the previous length.
    /// If there is no previous length, it does nothing.
    pub fn exit_function_call(&mut self) {
        if let Some(previous_stack_len) = self.previous_stack_lengths.pop() {
            self.stack.truncate(previous_stack_len);
        }
    }

    /// Creates new stack with the given function name.
    pub fn new_stack(&self, function_name: FunctionName) -> Vec<FunctionName> {
        let mut stack = self.stack.clone();

        let empty_or_different_function = self.stack.last().is_none_or(|current_function| {
            current_function.function_name() != function_name.function_name()
        });

        if empty_or_different_function {
            stack.push(function_name);
        }

        stack
    }
}
