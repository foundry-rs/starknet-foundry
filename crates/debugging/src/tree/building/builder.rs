use ptree::TreeBuilder;
use ptree::item::StringItem;

/// A wrapper around [`TreeBuilder`] that adds two guard nodes to the tree upon creation and removes them upon building.
pub struct TreeBuilderWithGuard {
    builder: TreeBuilder,
}

impl TreeBuilderWithGuard {
    /// Creates a new [`TreeBuilderWithGuard`].
    pub fn new() -> Self {
        let mut builder = TreeBuilder::new("guard".to_string());
        builder.begin_child("guard".to_string());
        Self { builder }
    }

    /// Add a child to the current item and make the new child current.
    pub fn begin_child(&mut self, text: String) {
        self.builder.begin_child(text);
    }

    /// Add an empty child (leaf item) to the current item.
    pub fn add_empty_child(&mut self, text: String) {
        self.builder.add_empty_child(text);
    }

    /// Finish adding children, and make the current item's parent current.
    pub fn end_child(&mut self) {
        self.builder.end_child();
    }

    /// Finish building the tree and return the top level item, not accounting for the guard nodes.
    pub fn build(mut self) -> StringItem {
        let string_item = self.builder.build();
        let string_item = extract_guard(string_item);
        extract_guard(string_item)
    }
}

/// Extracts the guard node from a [`StringItem`].
fn extract_guard(mut string_item: StringItem) -> StringItem {
    assert_eq!(string_item.text, "guard");
    assert_eq!(string_item.children.len(), 1);
    string_item.children.pop().unwrap_or_else(|| {
        unreachable!("guard is guaranteed to have one child by the assertion above")
    })
}
