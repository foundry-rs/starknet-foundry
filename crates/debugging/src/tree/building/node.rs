use crate::tree::building::builder::TreeBuilderWithGuard;
use crate::tree::ui::as_tree_node::AsTreeNode;
use crate::tree::ui::display::NodeDisplay;

/// Abstraction over a [`ptree::TreeBuilder`] that automatically ends building a current node when dropped.
/// # Note
/// The last node should be at level one when dropped, to not cause overflow.
/// This is enforced at type level by using [`TreeBuilderWithGuard`] instead of [`ptree::TreeBuilder`].
pub struct Node<'a> {
    builder: &'a mut TreeBuilderWithGuard,
}

impl Drop for Node<'_> {
    fn drop(&mut self) {
        self.builder.end_child();
    }
}

impl<'a> Node<'a> {
    /// Creates a new [`Node`] with a given [`TreeBuilderWithGuard`].
    pub fn new(builder: &'a mut TreeBuilderWithGuard) -> Self {
        Node { builder }
    }

    /// Calls [`AsTreeNode::as_tree_node`] on the given item.
    /// Utility function to allow chaining.
    pub fn as_tree_node(&mut self, tree_item: &impl AsTreeNode) {
        tree_item.as_tree_node(self);
    }

    /// Calls [`AsTreeNode::as_tree_node`] on the given item if it is not `None`.
    pub fn as_tree_node_optional(&mut self, tree_item: Option<&impl AsTreeNode>) {
        if let Some(tree_item) = tree_item {
            self.as_tree_node(tree_item);
        }
    }

    /// Creates a child node which parent is the current node and returns handle to created node.
    #[must_use = "if you want to create a leaf node use leaf() instead"]
    pub fn child_node(&mut self, tree_item: &impl NodeDisplay) -> Node<'_> {
        self.builder.begin_child(tree_item.display());
        Node::new(self.builder)
    }

    /// Creates a leaf node which parent is the current node.
    pub fn leaf(&mut self, tree_item: &impl NodeDisplay) {
        self.builder.add_empty_child(tree_item.display());
    }

    /// Creates a leaf node which parent is the current node if the item is not `None`.
    pub fn leaf_optional(&mut self, tree_item: Option<&impl NodeDisplay>) {
        if let Some(tree_item) = tree_item {
            self.leaf(tree_item);
        }
    }
}
