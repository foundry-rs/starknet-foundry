#[derive(Drop, Serde)]
pub struct AvailableResourceBoundsConfig {
    pub l1_gas: felt252,
    pub l1_data_gas: felt252,
    pub l2_gas: felt252,
}

#[derive(Drop, Serde)]
pub enum AvailableGasConfig {
    MaxGas: felt252,
    MaxResourceBounds: AvailableResourceBoundsConfig,
}

#[derive(Drop, Serde)]
pub enum BlockId {
    BlockTag: (),
    BlockHash: felt252,
    BlockNumber: felt252,
}

#[derive(Drop, Serde)]
pub struct InlineForkConfig {
    pub url: ByteArray,
    pub block: BlockId,
}

#[derive(Drop, Serde)]
pub struct OverriddenForkConfig {
    pub name: ByteArray,
    pub block: BlockId,
}

#[derive(Drop, Serde)]
pub enum ForkConfig {
    Inline: InlineForkConfig,
    Named: ByteArray,
    Overridden: OverriddenForkConfig,
}

#[derive(Drop, Serde)]
pub struct FuzzerConfig {
    pub runs: Option<felt252>,
    pub seed: Option<felt252>,
}

#[derive(Drop, Serde)]
pub enum Expected {
    ShortString: felt252,
    ByteArray: ByteArray,
    Array: Array<felt252>,
    Any,
}

#[derive(Drop, Serde)]
pub struct ShouldPanicConfig {
    pub expected: Expected,
}

#[derive(Drop, Serde)]
pub struct IgnoreConfig {
    pub is_ignored: bool,
}
