use crate::trace::function::name::FunctionName;

/// Represents a node in the function call tree, where each node corresponds to a function name
/// and can have child nodes representing nested function calls.
#[derive(Debug, Clone)]
pub struct FunctionNode {
    pub value: FunctionName,
    pub children: Vec<FunctionNode>,
}

impl FunctionNode {
    /// Creates a new [`FunctionNode`] with the given [`FunctionName`].
    pub fn new(value: FunctionName) -> Self {
        FunctionNode {
            value,
            children: Vec::new(),
        }
    }

    /// Adds a path of function names to the current node, creating child nodes as necessary.
    pub fn add_path(&mut self, path: Vec<FunctionName>) {
        self.add_path_recursive(&mut path.into_iter());
    }

    fn add_path_recursive(&mut self, iter: &mut impl Iterator<Item = FunctionName>) {
        if let Some(next) = iter.next() {
            if let Some(child) = self.children.iter_mut().find(|c| c.value == next) {
                child.add_path_recursive(iter);
            } else {
                let mut new_child = FunctionNode::new(next);
                new_child.add_path_recursive(iter);
                self.children.push(new_child);
            }
        }
    }
}
