#[derive(Drop)]
struct AvailableGasConfig {
    gas: felt252
}

#[derive(Drop)]
enum BlockId {
    BlockTag: (),
    BlockHash: felt252,
    BlockNumber: felt252
}

#[derive(Drop)]
struct ForkConfig {
    url: ByteArray,
    block: BlockId
}

#[derive(Drop)]
struct FuzzerConfig {
    runs: Option<felt252>,
    seed: Option<felt252>
}

#[derive(Drop)]
enum String {
    Short: felt252,
    Normal: ByteArray
}

#[derive(Drop)]
struct ShouldPanicConfig {
    expected: Array<String>,
}

#[derive(Drop)]
struct IgnoreConfig {
    is_ignored: bool,
}

#[derive(Drop)]
struct TestConfig {
    gas: Option<AvailableGasConfig>,
    fork: Option<ForkConfig>,
    fuzzer: Option<FuzzerConfig>,
    should_panic: Option<ShouldPanicConfig>,
    ignore: Option<IgnoreConfig>,
}
