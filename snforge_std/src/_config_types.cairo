#[derive(Drop, Serde)]
struct AvailableGasConfig {
    gas: felt252
}

#[derive(Drop, Serde)]
enum BlockId {
    BlockTag: (),
    BlockHash: felt252,
    BlockNumber: felt252
}

#[derive(Drop, Serde)]
struct InlineForkConfig {
    url: ByteArray,
    block: BlockId
}

struct MixedForkConfig {
    name: ByteArray,
    block: BlockId
}

#[derive(Drop, Serde)]
enum ForkConfig {
    Inline: InlineForkConfig,
    Named: ByteArray
    Mixed: MixedForkConfig
}

#[derive(Drop, Serde)]
struct FuzzerConfig {
    runs: Option<felt252>,
    seed: Option<felt252>
}

#[derive(Drop, Serde)]
enum Expected {
    ShortString: felt252,
    ByteArray: ByteArray,
    Array: Array<felt252>,
    Any
}

#[derive(Drop, Serde)]
struct ShouldPanicConfig {
    expected: Expected,
}

#[derive(Drop, Serde)]
struct IgnoreConfig {
    is_ignored: bool,
}
