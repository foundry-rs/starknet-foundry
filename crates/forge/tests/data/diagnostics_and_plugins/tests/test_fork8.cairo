#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0.4", block_id: BlockId::Tag(Random))]
fn incorrect_fork_attributes8() {
    assert(1 == 1, 'ok')
}
