#[test]
#[fork(url: "http://188.34.188.184:9545/rpc/v0.4", block_id: BlockId::Hash(Latest))]
fn incorrect_fork_attributes5() {
    assert(1 == 1, 'ok')
}
