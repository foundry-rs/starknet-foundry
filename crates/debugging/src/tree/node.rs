use crate::tree::display::NodeDisplay;
use crate::tree::serialize::TreeSerialize;
use ptree::TreeBuilder;
use ptree::item::StringItem;

/// Abstraction over a [`TreeBuilder`] that automatically ends building a current node when dropped.
pub struct Node<'a> {
    pub builder: &'a mut TreeBuilder,
}

impl Drop for Node<'_> {
    fn drop(&mut self) {
        self.builder.end_child();
    }
}

impl<'a> Node<'a> {
    /// Creates a [`TreeBuilder`] that should be used in [`Node::new`].
    /// ## Note:
    /// It is advised to use [`create_builder`] over creating a builder manually.
    /// The root [`Node`] should be at level 1 to not cause overflow when dropping.
    /// This can be achieved by creating a [`TreeBuilder`] with two guard nodes.
    pub fn create_builder() -> TreeBuilder {
        let mut builder = TreeBuilder::new("guard".to_string());
        builder.begin_child("guard".to_string());
        builder
    }

    /// Creates a new [`Node`] with a given [`TreeBuilder`].
    pub fn new(builder: &'a mut TreeBuilder) -> Self {
        Node { builder }
    }

    /// Serializes a given [`TreeSerialize`] item.
    /// Utility function to allow chaining.
    pub fn serialize(&mut self, tree_item: &impl TreeSerialize) {
        tree_item.serialize(self);
    }

    /// Creates a child node which parent is the current node and returns handle to created node.
    #[must_use = "if you want to create a leaf node use leaf() instead"]
    pub fn child_node(&mut self, tree_item: &impl NodeDisplay) -> Node {
        self.builder.begin_child(tree_item.display());
        Node::new(self.builder)
    }

    /// Creates a leaf node which parent is the current node.
    pub fn leaf(&mut self, tree_item: &impl NodeDisplay) {
        self.builder.add_empty_child(tree_item.display());
    }

    /// Consumes the [`Node`] and returns the serialized tree as a string.
    /// ## Note:
    /// This function assumes that the builder was created with [`Node::create_builder`].
    /// As it extracts two guard nodes from the tree, it will panic if the assumption is not met.
    pub fn into_string(self) -> String {
        let string_item = self.builder.build();

        let string_item = extract_guard(string_item);
        let string_item = extract_guard(string_item);

        write_to_string(&string_item)
    }
}

/// Writes a [`StringItem`] to a string.
fn write_to_string(string_item: &StringItem) -> String {
    let mut buf = Vec::new();
    ptree::write_tree(string_item, &mut buf).expect("write_tree failed");
    String::from_utf8(buf).expect("valid UTF-8")
}

/// Extracts the guard node from a [`StringItem`].
fn extract_guard(mut string_item: StringItem) -> StringItem {
    assert_eq!(string_item.text, "guard");
    assert_eq!(string_item.children.len(), 1);
    string_item.children.pop().unwrap_or_else(|| {
        unreachable!("guard is guaranteed to have one child by the assertion above")
    })
}
