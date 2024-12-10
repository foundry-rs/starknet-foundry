#[derive(Drop, Copy, Serde)]
pub enum BlockTag {
    Latest,
    Pending,
}

#[derive(Drop, Copy, Serde)]
pub enum BlockId {
    Tag: BlockTag,
    Hash: felt252,
    Number: u64,
}
