mod building;
mod ui;

use crate::tree::building::builder::TreeBuilderWithGuard;
use crate::tree::building::node::Node;
use crate::tree::ui::as_tree_node::AsTreeNode;
use ptree::item::StringItem;

/// Serialize a type that implements [`AsTreeNode`] to a string.
pub trait TreeSerialize {
    fn serialize(&self) -> String;
}

impl<T: AsTreeNode> TreeSerialize for T {
    fn serialize(&self) -> String {
        let mut builder = TreeBuilderWithGuard::new();
        Node::new(&mut builder).as_tree_node(self);
        let string_item = builder.build();
        write_to_string(&string_item)
    }
}

/// Writes a [`StringItem`] to a string.
fn write_to_string(string_item: &StringItem) -> String {
    let mut buf = Vec::new();
    ptree::write_tree(string_item, &mut buf).expect("write_tree failed");
    String::from_utf8(buf).expect("valid UTF-8")
}
