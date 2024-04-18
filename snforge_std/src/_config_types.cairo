struct AvailableGasConfig {
    gas: felt252
}

enum BlockId {
    BlockTag: (),
    BlockHash: felt252,
    BlockNumber: felt252
}

struct ForkConfig {
    url: ByteArray,
    block: BlockId
}

struct FuzzerConfig {
    runs: felt252,
    seed: felt252
}

enum String {
    Short: felt252,
    Normal: ByteArray
}

struct ShouldPanicConfig {
    expected: Array<String>,
}

struct IgnoreConfig {
    is_ignored: bool,
}

struct TestConfig {
    gas: Option<AvailableGasConfig>,
    fork: Option<ForkConfig>,
    fuzzer: Option<FuzzerConfig>,
    should_panic: Option<ShouldPanicConfig>,
    ignore: Option<IgnoreConfig>,
}
