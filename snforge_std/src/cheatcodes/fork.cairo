use serde::Serde;


#[derive(Drop, Copy, Serde)]
enum BlockTag {
    Latest,
    Pending,
}

#[derive(Drop, Copy, Serde)]
enum BlockId {
    Tag: BlockTag,
    Hash: felt252,
    Number: felt252,
}
